//! Reading bookmarks back, and small edits to a single bookmark.

use std::{collections::HashSet, path::PathBuf};

use knoa_core::{
    Bookmark, BookmarkId, FolderId, FolderLink, Image, Kind, Origin, Place, Query, Status,
    WorkspaceId,
};
use rusqlite::{OptionalExtension as _, Row, params_from_iter};

use crate::{Result, Store, search::Filter, search::kind_from_column, sync::Recorder};

/// Badge numbers for the sidebar's places.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    pub everything: usize,
    pub up_next: usize,
    pub unsorted: usize,
    pub not_organized: usize,
    pub set_aside: usize,
    pub trash: usize,
    /// Queued or being organized right now: the AI activity indicator.
    pub organizing: usize,
}

const COLUMNS: &str = "b.id, b.url, b.title, b.site, b.added_at, b.kind, b.summary, b.fields, b.extras, b.status,
    b.up_next, b.image_path, b.image_width, b.image_height, b.browser_folder, b.removed_from_browser, b.model,
    (SELECT json_group_array(json_array(tag, origin)) FROM bookmark_tags WHERE bookmark_id = b.id),
    (SELECT json_group_array(json_array(folder_id, origin, probability)) FROM bookmark_folders WHERE bookmark_id = b.id),
    b.workspace_id, b.set_aside, b.deleted_at";

impl Store {
    /// Bookmarks matching `query`, best match first when there is text, newest first otherwise.
    pub fn search(&self, query: &Query, limit: usize) -> Result<Vec<Bookmark>> {
        let filter = Filter::new(query);
        let sql = format!(
            "SELECT {COLUMNS} {} {} LIMIT {limit}",
            filter.sql,
            filter.order_by()
        );
        let mut stmt = self.conn.prepare_cached(&sql)?;
        let rows = stmt.query_map(params_from_iter(&filter.params), RawBookmark::from_row)?;
        rows.map(|row| row?.into_bookmark()).collect()
    }

    pub fn bookmark(&self, id: BookmarkId) -> Result<Option<Bookmark>> {
        let sql = format!("SELECT {COLUMNS} FROM bookmarks b WHERE b.id = ?1");
        let raw = self
            .conn
            .query_row(&sql, [id.0], RawBookmark::from_row)
            .optional()?;
        raw.map(RawBookmark::into_bookmark).transpose()
    }

    /// How many results each kind has for `query` with its kind filter removed: the kind chips.
    pub fn kind_counts(&self, query: &Query) -> Result<Vec<(Kind, usize)>> {
        let query = Query {
            kind: None,
            ..query.clone()
        };
        let filter = Filter::new(&query);
        let sql = format!("SELECT b.kind, count(*) {} GROUP BY b.kind", filter.sql);
        let mut stmt = self.conn.prepare_cached(&sql)?;
        let rows = stmt.query_map(params_from_iter(&filter.params), |row| {
            Ok((
                kind_from_column(row.get::<_, Option<String>>(0)?.as_deref()),
                row.get::<_, usize>(1)?,
            ))
        })?;
        let mut counts: Vec<(Kind, usize)> = rows
            .filter_map(|row| row.map(|(kind, n)| kind.map(|k| (k, n))).transpose())
            .collect::<rusqlite::Result<_>>()?;
        counts.sort_by_key(|(kind, _)| Kind::ALL.iter().position(|k| k == kind));
        Ok(counts)
    }

    pub fn count(&self, query: &Query) -> Result<usize> {
        let filter = Filter::new(query);
        let sql = format!("SELECT count(*) {}", filter.sql);
        Ok(self
            .conn
            .query_row(&sql, params_from_iter(&filter.params), |row| row.get(0))?)
    }

    /// The workspace's place counts. The AI indicator counts every workspace's queue.
    pub fn counts(&self, workspace: WorkspaceId) -> Result<Counts> {
        let place = |place| {
            self.count(&Query {
                workspace: Some(workspace),
                place,
                ..Query::default()
            })
        };
        let organizing = self.conn.query_row(
            "SELECT count(*) FROM bookmarks WHERE status IN ('queued', 'organizing')",
            [],
            |row| row.get(0),
        )?;
        Ok(Counts {
            everything: place(Place::Everything)?,
            up_next: place(Place::UpNext)?,
            unsorted: place(Place::Unsorted)?,
            not_organized: place(Place::NotOrganized)?,
            set_aside: place(Place::SetAside)?,
            trash: place(Place::Trash)?,
            organizing,
        })
    }

    /// Adds a bookmark to, or removes it from, Up next. Only the user does this.
    pub fn set_up_next(&mut self, id: BookmarkId, up_next: bool) -> Result<()> {
        self.edit_bookmark(
            id,
            "UPDATE bookmarks SET up_next = ?2 WHERE id = ?1",
            rusqlite::params![id.0, up_next],
        )
    }

    /// Points a bookmark at a thumbnail this Mac just fetched. The old file is left to the
    /// launch sweep, since other bookmarks may share it. The server's image no longer
    /// matches the cached one, so a linked library uploads the new one.
    pub fn set_image(&mut self, id: BookmarkId, image: &Image) -> Result<()> {
        self.conn.execute(
            "UPDATE bookmarks SET image_path = ?2, image_width = ?3, image_height = ?4,
                image_key = NULL, pending_image = hex(randomblob(16))
             WHERE id = ?1",
            rusqlite::params![
                id.0,
                image.path.to_string_lossy(),
                image.width,
                image.height
            ],
        )?;
        Ok(())
    }

    /// Every thumbnail a bookmark points to, in Trash or not. Several bookmarks can share one.
    pub fn image_paths(&self) -> Result<HashSet<PathBuf>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT image_path FROM bookmarks WHERE image_path IS NOT NULL")?;
        let paths = stmt.query_map([], |row| row.get::<_, String>(0))?;
        Ok(paths
            .map(|path| path.map(PathBuf::from))
            .collect::<rusqlite::Result<_>>()?)
    }

    /// Keeps a bookmark the AI set aside. The guideline never sets it aside again.
    pub fn keep(&mut self, id: BookmarkId) -> Result<()> {
        self.edit_bookmark(
            id,
            "UPDATE bookmarks SET kept = 1, set_aside = NULL WHERE id = ?1",
            [id.0],
        )
    }

    /// Runs a one-statement edit the user made to bookmark `id`.
    fn edit_bookmark(
        &mut self,
        id: BookmarkId,
        sql: &str,
        params: impl rusqlite::Params,
    ) -> Result<()> {
        let tx = self.conn.transaction()?;
        Recorder::user(&tx)?.on_bookmark(id.0, || {
            tx.execute(sql, params)?;
            Ok(())
        })?;
        tx.commit()?;
        Ok(())
    }
}

/// A row as stored, before the JSON columns are parsed.
struct RawBookmark {
    id: i64,
    workspace: i64,
    url: String,
    title: String,
    site: String,
    added_at: i64,
    kind: Option<String>,
    summary: Option<String>,
    fields: String,
    extras: String,
    status: String,
    up_next: bool,
    image: Option<(String, u32, u32)>,
    browser_folder: Option<String>,
    removed_from_browser: bool,
    model: Option<String>,
    tags: String,
    folders: String,
    set_aside: Option<String>,
    deleted_at: Option<i64>,
}

impl RawBookmark {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<RawBookmark> {
        let image_path: Option<String> = row.get(11)?;
        Ok(RawBookmark {
            id: row.get(0)?,
            workspace: row.get(19)?,
            url: row.get(1)?,
            title: row.get(2)?,
            site: row.get(3)?,
            added_at: row.get(4)?,
            kind: row.get(5)?,
            summary: row.get(6)?,
            fields: row.get(7)?,
            extras: row.get(8)?,
            status: row.get(9)?,
            up_next: row.get(10)?,
            image: match image_path {
                Some(path) => Some((path, row.get(12)?, row.get(13)?)),
                None => None,
            },
            browser_folder: row.get(14)?,
            removed_from_browser: row.get(15)?,
            model: row.get(16)?,
            tags: row.get(17)?,
            folders: row.get(18)?,
            set_aside: row.get(20)?,
            deleted_at: row.get(21)?,
        })
    }

    fn into_bookmark(self) -> Result<Bookmark> {
        let tags: Vec<(String, String)> = serde_json::from_str(&self.tags)?;
        let folders: Vec<(i64, String, Option<f32>)> = serde_json::from_str(&self.folders)?;
        Ok(Bookmark {
            id: BookmarkId(self.id),
            workspace: WorkspaceId(self.workspace),
            url: self.url,
            title: self.title,
            site: self.site,
            added_at: self.added_at,
            kind: kind_from_column(self.kind.as_deref()),
            summary: self.summary,
            fields: serde_json::from_str(&self.fields)?,
            extras: serde_json::from_str(&self.extras)?,
            tags: tags
                .into_iter()
                .map(|(tag, origin)| (tag, Origin::from_slug(&origin)))
                .collect(),
            folders: folders
                .into_iter()
                .map(|(id, origin, probability)| FolderLink {
                    folder: FolderId(id),
                    origin: Origin::from_slug(&origin),
                    probability,
                })
                .collect(),
            up_next: self.up_next,
            status: Status::from_slug(&self.status).unwrap_or(Status::New),
            image: self.image.map(|(path, width, height)| Image {
                path: PathBuf::from(path),
                width,
                height,
            }),
            browser_folder: self.browser_folder,
            removed_from_browser: self.removed_from_browser,
            set_aside: self.set_aside,
            deleted_at: self.deleted_at,
            model: self.model,
        })
    }
}

#[cfg(test)]
mod tests {
    use knoa_core::{Enrichment, Fields, Kind};
    use serde_json::json;

    use super::*;
    use crate::{
        NewBookmarks,
        test_support::{WORKSPACE, bookmark},
    };

    #[test]
    fn search_combines_prefix_text_with_field_filters() {
        let mut store = Store::open_in_memory().unwrap();
        let movies = [
            bookmark("https://imdb.com/1", "The Third Man", 1),
            bookmark("https://imdb.com/2", "The Thin Red Line", 2),
        ];
        store
            .import(WORKSPACE, "test", &movies, NewBookmarks::Organize)
            .unwrap();
        for (job, runtime) in store.take_jobs(0, 2).unwrap().into_iter().zip([170, 104]) {
            let mut fields = Fields::new();
            fields.insert("runtime".into(), json!(runtime));
            let enrichment = Enrichment {
                kind: Kind::Movie,
                summary: Some("A film noir".into()),
                fields,
                extras: Vec::new(),
                tags: Vec::new(),
                folders: Vec::new(),
                set_aside: None,
                model: "test".into(),
            };
            store
                .finish_organized(job.bookmark.id, &enrichment, 3)
                .unwrap();
        }

        let titles = |input: &str| -> Vec<String> {
            let found = store
                .search(&Query::parse(input, None, Place::Everything), 10)
                .unwrap();
            let mut titles: Vec<String> = found.into_iter().map(|b| b.title).collect();
            titles.sort();
            titles
        };
        assert_eq!(titles("th"), ["The Thin Red Line", "The Third Man"]);
        assert_eq!(titles("thi noir runtime>2h"), ["The Thin Red Line"]);
        assert_eq!(
            store
                .kind_counts(&Query::parse("noir", None, Place::Everything))
                .unwrap(),
            [(Kind::Movie, 2)]
        );
    }
}
