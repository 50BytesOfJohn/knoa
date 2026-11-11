//! Getting bookmarks in from a browser: idempotent imports.

use std::collections::HashSet;

use knoa_core::{NewBookmark, WorkspaceId, site_of};
use rusqlite::{OptionalExtension as _, params};

use crate::{Result, Store, search, sync::Recorder, trash};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub added: usize,
    pub removed: usize,
}

/// Whether newly imported bookmarks go straight into the organizing queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewBookmarks {
    /// The first import: store titles and URLs only, organizing is a separate action.
    MetadataOnly,
    Organize,
}

impl Store {
    /// Brings a workspace in line with a browser's current bookmarks.
    ///
    /// `source` names the browser profile. Bookmarks are keyed on workspace and URL, so
    /// importing the same list twice changes nothing. Bookmarks from `source` that are gone
    /// from `bookmarks` are marked as removed from the browser, never deleted. Bookmarks the
    /// user deleted stay deleted unless the browser has a new bookmark for the URL.
    pub fn import(
        &mut self,
        workspace: WorkspaceId,
        source: &str,
        bookmarks: &[NewBookmark],
        new: NewBookmarks,
    ) -> Result<ImportReport> {
        let tx = self.conn.transaction()?;
        let recorder = Recorder::user(&tx)?;
        let mut report = ImportReport::default();
        let mut seen = HashSet::new();
        for bookmark in bookmarks {
            if !seen.insert(bookmark.url.as_str()) {
                continue;
            }
            match upsert(&tx, &recorder, workspace, source, bookmark, new)? {
                Upserted::Added(id) => {
                    report.added += 1;
                    recorder.created_bookmark(id)?;
                    search::reindex(&tx, id)?;
                }
                Upserted::Updated(id) => search::reindex(&tx, id)?,
                Upserted::Unchanged | Upserted::DeletedForGood => {}
            }
        }
        report.removed = mark_removed(&tx, workspace, source, &seen)?;
        trash::forget_gone_from_browser(&tx, workspace, source, &seen)?;
        tx.commit()?;
        Ok(report)
    }
}

impl Store {
    /// Whether anything was ever imported from `source` into `workspace`; the first import
    /// is metadata-only.
    pub fn has_source(&self, workspace: WorkspaceId, source: &str) -> Result<bool> {
        let sql = "SELECT EXISTS (SELECT 1 FROM bookmarks WHERE workspace_id = ?1 AND source = ?2)";
        Ok(self
            .conn
            .query_row(sql, params![workspace.0, source], |row| row.get(0))?)
    }
}

/// What importing one browser bookmark did to the workspace.
enum Upserted {
    Added(i64),
    Updated(i64),
    Unchanged,
    /// The user deleted the URL for good and the browser hasn't bookmarked it again.
    DeletedForGood,
}

/// The browser-synced columns of a bookmark already in the workspace.
struct Stored {
    id: i64,
    /// What this Mac's browser last said; the synced title and folder may be another's.
    seen_title: Option<String>,
    seen_folder: Option<String>,
    source: Option<String>,
    guid: Option<String>,
    removed_from_browser: bool,
    deleted: bool,
}

fn upsert(
    tx: &rusqlite::Transaction<'_>,
    recorder: &Recorder<'_>,
    workspace: WorkspaceId,
    source: &str,
    bookmark: &NewBookmark,
    new: NewBookmarks,
) -> Result<Upserted> {
    if let Some(stored) = stored(tx, workspace, &bookmark.url)? {
        return recorder.on_bookmark(stored.id, || update(tx, &stored, source, bookmark));
    }
    let guid = bookmark.browser_guid.as_deref();
    if let Some((old_source, old_guid)) = trash::deleted_url(tx, workspace, &bookmark.url)? {
        if !trash::bookmarked_again(Some(&old_source), old_guid.as_deref(), source, guid) {
            return Ok(Upserted::DeletedForGood);
        }
        trash::forget_deleted_url(tx, workspace, &bookmark.url)?;
    }
    Ok(Upserted::Added(insert(
        tx, workspace, source, bookmark, new,
    )?))
}

fn stored(
    tx: &rusqlite::Transaction<'_>,
    workspace: WorkspaceId,
    url: &str,
) -> Result<Option<Stored>> {
    let sql = "SELECT id, seen_title, seen_folder, source, browser_guid, removed_from_browser,
                      deleted_at IS NOT NULL
               FROM bookmarks WHERE workspace_id = ?1 AND url = ?2";
    let stored = tx
        .query_row(sql, params![workspace.0, url], |row| {
            Ok(Stored {
                id: row.get(0)?,
                seen_title: row.get(1)?,
                seen_folder: row.get(2)?,
                source: row.get(3)?,
                guid: row.get(4)?,
                removed_from_browser: row.get(5)?,
                deleted: row.get(6)?,
            })
        })
        .optional()?;
    Ok(stored)
}

/// Every save in the browser re-imports its whole file, so a bookmark the browser didn't
/// change is left alone: rewriting it would reindex its page text while the store is
/// locked, for every bookmark in the library.
fn update(
    tx: &rusqlite::Transaction<'_>,
    stored: &Stored,
    source: &str,
    bookmark: &NewBookmark,
) -> Result<Upserted> {
    let guid = bookmark.browser_guid.as_deref();
    let again = trash::bookmarked_again(
        stored.source.as_deref(),
        stored.guid.as_deref(),
        source,
        guid,
    );
    let new_title = stored.seen_title.as_deref() != Some(bookmark.title.as_str());
    let new_folder = stored.seen_folder != bookmark.browser_folder;
    let unchanged = !(new_title || new_folder)
        && stored.source.as_deref() == Some(source)
        && stored.guid.as_deref() == guid
        && !stored.removed_from_browser
        && !(again && stored.deleted);
    if unchanged {
        return Ok(Upserted::Unchanged);
    }
    tx.execute(
        "UPDATE bookmarks SET title = CASE WHEN ?7 THEN ?2 ELSE title END, seen_title = ?2,
         browser_folder = CASE WHEN ?8 THEN ?5 ELSE browser_folder END, seen_folder = ?5,
         source = ?3, browser_guid = ?4, removed_from_browser = 0,
         deleted_at = CASE WHEN ?6 THEN NULL ELSE deleted_at END
         WHERE id = ?1",
        params![
            stored.id,
            bookmark.title,
            source,
            guid,
            bookmark.browser_folder,
            again,
            new_title,
            new_folder,
        ],
    )?;
    Ok(Upserted::Updated(stored.id))
}

fn insert(
    tx: &rusqlite::Transaction<'_>,
    workspace: WorkspaceId,
    source: &str,
    bookmark: &NewBookmark,
    new: NewBookmarks,
) -> Result<i64> {
    let status = match new {
        NewBookmarks::MetadataOnly => "new",
        NewBookmarks::Organize => "queued",
    };
    tx.execute(
        "INSERT INTO bookmarks (workspace_id, url, title, site, added_at, status, source, browser_guid,
                                browser_folder, seen_title, seen_folder)
         VALUES (?9, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?2, ?8)",
        params![
            bookmark.url,
            bookmark.title,
            site_of(&bookmark.url),
            bookmark.added_at,
            status,
            source,
            bookmark.browser_guid,
            bookmark.browser_folder,
            workspace.0
        ],
    )?;
    let id = tx.last_insert_rowid();
    if new == NewBookmarks::Organize {
        tx.execute("INSERT INTO jobs (bookmark_id) VALUES (?1)", [id])?;
    }
    Ok(id)
}

fn mark_removed(
    tx: &rusqlite::Transaction<'_>,
    workspace: WorkspaceId,
    source: &str,
    present: &HashSet<&str>,
) -> Result<usize> {
    let mut stmt = tx.prepare(
        "SELECT id, url FROM bookmarks WHERE workspace_id = ?1 AND source = ?2 AND removed_from_browser = 0",
    )?;
    let stored = stmt
        .query_map(params![workspace.0, source], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let gone: Vec<i64> = stored
        .into_iter()
        .filter(|(_, url)| !present.contains(url.as_str()))
        .map(|(id, _)| id)
        .collect();
    for id in &gone {
        tx.execute(
            "UPDATE bookmarks SET removed_from_browser = 1 WHERE id = ?1",
            [id],
        )?;
    }
    Ok(gone.len())
}

#[cfg(test)]
mod tests {
    use knoa_core::{BookmarkId, Ink, Place, Query};

    use super::*;
    use crate::test_support::{WORKSPACE, bookmark};

    #[test]
    fn reimport_is_idempotent_and_marks_removed_bookmarks() {
        let mut store = Store::open_in_memory().unwrap();
        let first = [
            bookmark("https://a.dev/", "A", 1),
            bookmark("https://b.dev/", "B", 2),
            bookmark("https://a.dev/", "A again", 3),
        ];
        let report = store
            .import(
                WORKSPACE,
                "helium:Default",
                &first,
                NewBookmarks::MetadataOnly,
            )
            .unwrap();
        assert_eq!(
            report,
            ImportReport {
                added: 2,
                removed: 0
            }
        );

        let report = store
            .import(
                WORKSPACE,
                "helium:Default",
                &first[..1],
                NewBookmarks::Organize,
            )
            .unwrap();
        assert_eq!(
            report,
            ImportReport {
                added: 0,
                removed: 1
            }
        );

        let all = store.search(&Query::default(), 10).unwrap();
        assert_eq!(all.len(), 2, "removed bookmarks stay in the library");
        let b = all.iter().find(|b| b.url == "https://b.dev/").unwrap();
        assert!(b.removed_from_browser);
        assert_eq!(
            store
                .count(&Query {
                    place: Place::NotOrganized,
                    ..Query::default()
                })
                .unwrap(),
            2
        );
        assert!(store.bookmark(BookmarkId(b.id.0)).unwrap().is_some());
    }

    #[test]
    fn reimport_writes_only_what_the_browser_changed() {
        let mut store = Store::open_in_memory().unwrap();
        let sync = |store: &mut Store, links: &[NewBookmark]| {
            store
                .import(WORKSPACE, "helium:Default", links, NewBookmarks::Organize)
                .unwrap();
        };
        let links = [
            bookmark("https://a.dev/", "A", 1),
            bookmark("https://b.dev/", "B", 2),
        ];
        sync(&mut store, &links);
        let before = store.conn.total_changes();
        sync(&mut store, &links);
        assert_eq!(
            store.conn.total_changes(),
            before,
            "nothing changed, nothing written"
        );

        sync(
            &mut store,
            &[bookmark("https://a.dev/", "Renamed", 1), links[1].clone()],
        );
        let found = store
            .search(&Query::parse("renamed", None, Place::Everything), 10)
            .unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].url, "https://a.dev/");
    }

    #[test]
    fn a_browser_feeds_each_workspace_its_own_copy() {
        let mut store = Store::open_in_memory().unwrap();
        let work = store.create_workspace("Work", Ink::Olive).unwrap();
        let links = [bookmark("https://a.dev/", "A", 1)];
        for workspace in [WORKSPACE, work] {
            store
                .import(workspace, "helium:Default", &links, NewBookmarks::Organize)
                .unwrap();
        }
        let in_work = Query {
            workspace: Some(work),
            ..Query::default()
        };
        assert_eq!(store.count(&Query::default()).unwrap(), 2);
        assert_eq!(store.count(&in_work).unwrap(), 1);

        store
            .import(work, "helium:Default", &[], NewBookmarks::Organize)
            .unwrap();
        let personal = store
            .search(
                &Query {
                    workspace: Some(WORKSPACE),
                    ..Query::default()
                },
                1,
            )
            .unwrap();
        assert!(!personal[0].removed_from_browser, "workspaces sync apart");
    }
}
