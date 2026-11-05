//! Trash: deleting bookmarks, restoring them, and deleting them for good.
//!
//! Sync is one-way, so a bookmark deleted here is usually still in the browser. Knoa keeps
//! it deleted anyway, until the browser drops it or the user bookmarks it again.

use std::collections::HashSet;

use knoa_core::{BookmarkId, TRASH_SECONDS, WorkspaceId};
use rusqlite::{OptionalExtension as _, Transaction, params};

use crate::{Result, Store};

impl Store {
    /// Moves a bookmark to Trash and takes it off the organizing queue.
    pub fn trash(&mut self, id: BookmarkId, now: i64) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "UPDATE bookmarks SET deleted_at = ?2,
                status = CASE WHEN status = 'queued' THEN 'new' ELSE status END
             WHERE id = ?1",
            params![id.0, now],
        )?;
        tx.execute("DELETE FROM jobs WHERE bookmark_id = ?1", [id.0])?;
        tx.commit()?;
        Ok(())
    }

    /// Takes a bookmark out of Trash, back to where it was.
    pub fn restore(&mut self, id: BookmarkId) -> Result<()> {
        self.conn.execute(
            "UPDATE bookmarks SET deleted_at = NULL WHERE id = ?1",
            [id.0],
        )?;
        Ok(())
    }

    /// Deletes one bookmark in Trash for good.
    pub fn delete_for_good(&mut self, id: BookmarkId) -> Result<()> {
        self.purge("id = ?1 AND deleted_at IS NOT NULL", id.0)?;
        Ok(())
    }

    /// Deletes everything in a workspace's Trash for good, and returns how many.
    pub fn empty_trash(&mut self, workspace: WorkspaceId) -> Result<usize> {
        self.purge("workspace_id = ?1 AND deleted_at IS NOT NULL", workspace.0)
    }

    /// Deletes for good, in every workspace, what has been in Trash for `TRASH_SECONDS`.
    pub fn purge_expired_trash(&mut self, now: i64) -> Result<usize> {
        self.purge("deleted_at <= ?1", now - TRASH_SECONDS)
    }

    /// Deletes the bookmarks matching `condition`, which takes one parameter.
    fn purge(&mut self, condition: &str, param: i64) -> Result<usize> {
        let tx = self.conn.transaction()?;
        tx.execute(
            &format!(
                "INSERT OR REPLACE INTO deleted_urls (workspace_id, url, source, browser_guid)
                 SELECT workspace_id, url, source, browser_guid FROM bookmarks
                 WHERE {condition} AND source IS NOT NULL AND removed_from_browser = 0"
            ),
            [param],
        )?;
        // The full-text index is a virtual table, outside the foreign-key cascade.
        tx.execute(
            &format!(
                "DELETE FROM bookmarks_fts WHERE rowid IN (SELECT id FROM bookmarks WHERE {condition})"
            ),
            [param],
        )?;
        let deleted = tx.execute(&format!("DELETE FROM bookmarks WHERE {condition}"), [param])?;
        tx.commit()?;
        Ok(deleted)
    }
}

/// Whether a browser bookmark is a new one for a URL the user deleted: the same profile has
/// it under a bookmark id it didn't have before. Without ids, it counts as the old one.
pub(crate) fn bookmarked_again(
    deleted_source: Option<&str>,
    deleted_guid: Option<&str>,
    source: &str,
    guid: Option<&str>,
) -> bool {
    deleted_source == Some(source)
        && matches!((deleted_guid, guid), (Some(old), Some(new)) if old != new)
}

/// The profile and bookmark id of a URL the user deleted for good, if any.
pub(crate) fn deleted_url(
    tx: &Transaction<'_>,
    workspace: WorkspaceId,
    url: &str,
) -> Result<Option<(String, Option<String>)>> {
    Ok(tx
        .query_row(
            "SELECT source, browser_guid FROM deleted_urls WHERE workspace_id = ?1 AND url = ?2",
            params![workspace.0, url],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?)
}

pub(crate) fn forget_deleted_url(
    tx: &Transaction<'_>,
    workspace: WorkspaceId,
    url: &str,
) -> Result<()> {
    tx.execute(
        "DELETE FROM deleted_urls WHERE workspace_id = ?1 AND url = ?2",
        params![workspace.0, url],
    )?;
    Ok(())
}

/// Forgets deleted URLs that `source` no longer has: bookmarking one again later is new.
pub(crate) fn forget_gone_from_browser(
    tx: &Transaction<'_>,
    workspace: WorkspaceId,
    source: &str,
    present: &HashSet<&str>,
) -> Result<()> {
    let urls: Vec<String> = {
        let mut stmt =
            tx.prepare("SELECT url FROM deleted_urls WHERE workspace_id = ?1 AND source = ?2")?;
        stmt.query_map(params![workspace.0, source], |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    for url in urls.iter().filter(|url| !present.contains(url.as_str())) {
        forget_deleted_url(tx, workspace, url)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use knoa_core::{NewBookmark, Place, Query};

    use super::*;
    use crate::{
        NewBookmarks,
        test_support::{WORKSPACE, bookmark},
    };

    fn with_guid(guid: &str) -> NewBookmark {
        NewBookmark {
            browser_guid: Some(guid.into()),
            ..bookmark("https://a.dev/", "A", 1)
        }
    }

    #[test]
    fn sync_brings_a_deleted_bookmark_back_only_when_bookmarked_again() {
        let mut store = Store::open_in_memory().unwrap();
        let sync = |store: &mut Store, links: &[NewBookmark]| {
            store
                .import(WORKSPACE, "helium:Default", links, NewBookmarks::Organize)
                .unwrap();
        };
        let count = |store: &Store, place| {
            store
                .count(&Query {
                    place,
                    ..Query::default()
                })
                .unwrap()
        };
        sync(&mut store, &[with_guid("one")]);
        let id = store.search(&Query::default(), 1).unwrap()[0].id;

        store.trash(id, 10).unwrap();
        sync(&mut store, &[with_guid("one")]);
        assert_eq!(count(&store, Place::Everything), 0);
        assert_eq!(count(&store, Place::Trash), 1);

        store.empty_trash(WORKSPACE).unwrap();
        sync(&mut store, &[with_guid("one")]);
        assert_eq!(count(&store, Place::Trash), 0);
        assert_eq!(count(&store, Place::Everything), 0, "still deleted");

        sync(&mut store, &[with_guid("two")]);
        assert_eq!(count(&store, Place::Everything), 1, "bookmarked again");
    }
}
