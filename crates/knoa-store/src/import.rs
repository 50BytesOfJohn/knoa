//! Idempotent import of browser bookmarks.

use std::collections::HashSet;

use knoa_core::{NewBookmark, site_of};
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

impl Store {
    /// Brings the library in line with a browser's current bookmarks.
    ///
    /// `source` names the browser profile. Bookmarks are keyed on URL, so importing the same
    /// list twice changes nothing. Bookmarks from `source` that are gone from `bookmarks` are
    /// marked as removed from the browser, never deleted.
    pub fn import(
        &mut self,
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
                    "SELECT id FROM bookmarks WHERE url = ?1",
                    [&bookmark.url],
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
                insert(&tx, source, bookmark, new)?
            };
            search::reindex(&tx, id)?;
        }
        report.removed = mark_removed(&tx, source, &seen)?;
        tx.commit()?;
        Ok(report)
    }
}

fn insert(
    tx: &rusqlite::Transaction<'_>,
    source: &str,
    bookmark: &NewBookmark,
    new: NewBookmarks,
) -> Result<i64> {
    let status = match new {
        NewBookmarks::MetadataOnly => "new",
        NewBookmarks::Organize => "queued",
    };
    tx.execute(
        "INSERT INTO bookmarks (url, title, site, added_at, status, source, browser_guid, browser_folder)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            bookmark.url,
            bookmark.title,
            site_of(&bookmark.url),
            bookmark.added_at,
            status,
            source,
            bookmark.browser_guid,
            bookmark.browser_folder
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
    source: &str,
    present: &HashSet<&str>,
) -> Result<usize> {
    let mut stmt =
        tx.prepare("SELECT id, url FROM bookmarks WHERE source = ?1 AND removed_from_browser = 0")?;
    let stored = stmt
        .query_map([source], |row| {
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
    use knoa_core::{BookmarkId, Place, Query};

    use super::*;
    use crate::test_support::bookmark;

    #[test]
    fn reimport_is_idempotent_and_marks_removed_bookmarks() {
        let mut store = Store::open_in_memory().unwrap();
        let first = [
            bookmark("https://a.dev/", "A", 1),
            bookmark("https://b.dev/", "B", 2),
            bookmark("https://a.dev/", "A again", 3),
        ];
        let report = store
            .import("helium:Default", &first, NewBookmarks::MetadataOnly)
            .unwrap();
        assert_eq!(
            report,
            ImportReport {
                added: 2,
                removed: 0
            }
        );

        let report = store
            .import("helium:Default", &first[..1], NewBookmarks::Organize)
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
}
