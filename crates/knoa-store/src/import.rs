//! Getting bookmarks in: idempotent browser imports and links added by hand.

use std::collections::HashSet;

use knoa_core::{BookmarkId, NewBookmark, WorkspaceId, site_of};
use rusqlite::{OptionalExtension as _, params};

use crate::{Result, Store, search};

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

/// A link the user added by hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddedLink {
    pub id: BookmarkId,
    /// False when the workspace already had this URL.
    pub is_new: bool,
}

impl Store {
    /// Brings a workspace in line with a browser's current bookmarks.
    ///
    /// `source` names the browser profile. Bookmarks are keyed on workspace and URL, so
    /// importing the same list twice changes nothing. Bookmarks from `source` that are gone
    /// from `bookmarks` are marked as removed from the browser, never deleted.
    pub fn import(
        &mut self,
        workspace: WorkspaceId,
        source: &str,
        bookmarks: &[NewBookmark],
        new: NewBookmarks,
    ) -> Result<ImportReport> {
        let tx = self.conn.transaction()?;
        let mut report = ImportReport::default();
        let mut seen = HashSet::new();
        for bookmark in bookmarks {
            if !seen.insert(bookmark.url.as_str()) {
                continue;
            }
            let existing: Option<i64> = tx
                .query_row(
                    "SELECT id FROM bookmarks WHERE workspace_id = ?1 AND url = ?2",
                    params![workspace.0, bookmark.url],
                    |row| row.get(0),
                )
                .optional()?;
            let id = if let Some(id) = existing {
                tx.execute(
                    "UPDATE bookmarks SET title = ?2, source = ?3, browser_guid = ?4, browser_folder = ?5,
                     removed_from_browser = 0 WHERE id = ?1",
                    params![id, bookmark.title, source, bookmark.browser_guid, bookmark.browser_folder],
                )?;
                id
            } else {
                report.added += 1;
                insert(&tx, workspace, source, bookmark, new)?
            };
            search::reindex(&tx, id)?;
        }
        report.removed = mark_removed(&tx, workspace, source, &seen)?;
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

    /// Adds a link by hand and queues it for organizing. The user chose it, so the
    /// workspace's guideline never sets it aside. The URL stands in as the title until the
    /// page is fetched.
    pub fn add_link(&mut self, workspace: WorkspaceId, url: &str, now: i64) -> Result<AddedLink> {
        let tx = self.conn.transaction()?;
        let existing: Option<i64> = tx
            .query_row(
                "SELECT id FROM bookmarks WHERE workspace_id = ?1 AND url = ?2",
                params![workspace.0, url],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(id) = existing {
            tx.execute(
                "UPDATE bookmarks SET kept = 1, set_aside = NULL WHERE id = ?1",
                [id],
            )?;
            tx.commit()?;
            return Ok(AddedLink {
                id: BookmarkId(id),
                is_new: false,
            });
        }
        tx.execute(
            "INSERT INTO bookmarks (workspace_id, url, title, site, added_at, status, kept)
             VALUES (?1, ?2, ?2, ?3, ?4, 'queued', 1)",
            params![workspace.0, url, site_of(url), now],
        )?;
        let id = tx.last_insert_rowid();
        tx.execute("INSERT INTO jobs (bookmark_id) VALUES (?1)", [id])?;
        search::reindex(&tx, id)?;
        tx.commit()?;
        Ok(AddedLink {
            id: BookmarkId(id),
            is_new: true,
        })
    }
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
        "INSERT INTO bookmarks (workspace_id, url, title, site, added_at, status, source, browser_guid, browser_folder)
         VALUES (?9, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
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
    use knoa_core::{BookmarkId, Enrichment, Kind, Place, Query};

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
    fn a_browser_feeds_each_workspace_its_own_copy() {
        let mut store = Store::open_in_memory().unwrap();
        let work = store.create_workspace("Work", 1).unwrap();
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

    #[test]
    fn a_kept_link_is_never_set_aside() {
        let mut store = Store::open_in_memory().unwrap();
        let added = store.add_link(WORKSPACE, "https://a.dev/", 1).unwrap();
        let job = store.take_jobs(1, 1).unwrap().remove(0);
        assert_eq!(job.bookmark.id, added.id);
        let enrichment = Enrichment {
            kind: Kind::Article,
            summary: None,
            fields: knoa_core::Fields::new(),
            extras: Vec::new(),
            tags: Vec::new(),
            folders: Vec::new(),
            set_aside: Some("not about work".into()),
            model: "test".into(),
        };
        store.finish_organized(added.id, &enrichment, 2).unwrap();
        let set_aside = Query {
            place: Place::SetAside,
            ..Query::default()
        };
        assert_eq!(store.count(&set_aside).unwrap(), 0);
        assert_eq!(store.count(&Query::default()).unwrap(), 1);
    }
}
