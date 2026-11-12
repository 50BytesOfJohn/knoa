//! Links the user adds by hand.

use knoa_core::{BookmarkId, WorkspaceId, site_of};
use rusqlite::{OptionalExtension as _, params};

use crate::{Result, Store, search, sync::Recorder, trash};

/// A link the user added by hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddedLink {
    pub id: BookmarkId,
    /// False when the workspace already had this URL.
    pub is_new: bool,
}

impl Store {
    /// Adds a link by hand and queues it for organizing. The user chose it, so the
    /// workspace's guideline never sets it aside, and a deleted copy comes back. The URL
    /// stands in as the title until the page is fetched.
    pub fn add_link(&mut self, workspace: WorkspaceId, url: &str, now: i64) -> Result<AddedLink> {
        let tx = self.conn.transaction()?;
        let existing: Option<i64> = tx
            .query_row(
                "SELECT id FROM bookmarks WHERE workspace_id = ?1 AND url = ?2",
                params![workspace.0, url],
                |row| row.get(0),
            )
            .optional()?;
        let recorder = Recorder::user(&tx)?;
        if let Some(id) = existing {
            recorder.on_bookmark(id, || {
                tx.execute(
                    "UPDATE bookmarks SET kept = 1, set_aside = NULL, deleted_at = NULL WHERE id = ?1",
                    [id],
                )?;
                Ok(())
            })?;
            tx.commit()?;
            return Ok(AddedLink {
                id: BookmarkId(id),
                is_new: false,
            });
        }
        trash::forget_deleted_url(&tx, workspace, url)?;
        tx.execute(
            "INSERT INTO bookmarks (workspace_id, url, title, site, added_at, status, kept)
             VALUES (?1, ?2, ?2, ?3, ?4, 'queued', 1)",
            params![workspace.0, url, site_of(url), now],
        )?;
        let id = tx.last_insert_rowid();
        tx.execute("INSERT INTO jobs (bookmark_id) VALUES (?1)", [id])?;
        recorder.created_bookmark(id)?;
        search::reindex(&tx, id)?;
        tx.commit()?;
        Ok(AddedLink {
            id: BookmarkId(id),
            is_new: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use knoa_core::{Enrichment, Kind, Place, Query};

    use super::*;
    use crate::test_support::WORKSPACE;

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
