//! Cloud sync's local side: sync ids, the outbox of local changes, linking the library to
//! an account, and applying what the server sends.
//!
//! Until the library is linked nothing here does anything: no row has a sync id and no
//! change is recorded, so Local mode is the app as it always was.

mod apply;
mod delivery;
mod device;
mod ids;
mod link;
mod outbox;
mod recorder;
mod rows;
mod snapshot;
mod unsent;

use std::collections::BTreeMap;

use knoa_core::{
    WorkspaceId,
    protocol::{MAX_PUSH, MAX_PUSH_BYTES, Mutation},
};
use rusqlite::{Connection, OptionalExtension as _};

pub(crate) use apply::apply_held;
pub use device::{CloudJob, ThumbnailDownload, ThumbnailUpload};
pub use link::AccountRows;
pub(crate) use recorder::Recorder;
pub(crate) use rows::page_text;
pub use unsent::Settled;

use crate::{Result, Store};

impl Store {
    /// Cloud ids for the current workspaces; local-only workspaces have no entry.
    pub fn workspace_sync_ids(&self) -> Result<BTreeMap<WorkspaceId, String>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT id, sync_id FROM workspaces WHERE sync_id IS NOT NULL")?;
        let rows = stmt.query_map([], |row| Ok((WorkspaceId(row.get(0)?), row.get(1)?)))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Whether the library is linked to an account.
    pub fn is_linked(&self) -> Result<bool> {
        linked(&self.conn)
    }

    /// The user id of the linked account.
    pub fn account(&self) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT user_id FROM sync_state", [], |row| row.get(0))
            .optional()?)
    }

    /// Where the next pull starts; `None` while not linked.
    pub fn cursor(&self) -> Result<Option<i64>> {
        Ok(self
            .conn
            .query_row("SELECT cursor FROM sync_state", [], |row| row.get(0))
            .optional()?)
    }

    /// The next push: the oldest local changes not pushed yet, in the order they were
    /// made, at most `MAX_PUSH` of them and `MAX_PUSH_BYTES` in all.
    pub fn next_push(&self) -> Result<Vec<Mutation>> {
        self.next_push_at_most(MAX_PUSH)
    }

    /// [`Store::next_push`], with at most `limit` changes.
    pub fn next_push_at_most(&self, limit: usize) -> Result<Vec<Mutation>> {
        outbox::pending(&self.conn, limit.min(MAX_PUSH), MAX_PUSH_BYTES)
    }
}

pub(crate) fn linked(conn: &Connection) -> Result<bool> {
    Ok(
        conn.query_row("SELECT EXISTS (SELECT 1 FROM sync_state)", [], |row| {
            row.get(0)
        })?,
    )
}

#[cfg(test)]
mod tests {
    use knoa_core::Ink;

    use super::*;
    use crate::test_support::{WORKSPACE, WORKSPACE_KEY, linked, pushed};

    #[test]
    fn workspace_cloud_ids_follow_linking_creation_and_deletion() {
        let mut store = Store::open_in_memory().unwrap();
        assert!(store.workspace_sync_ids().unwrap().is_empty());
        linked(&mut store);
        assert_eq!(
            store.workspace_sync_ids().unwrap()[&WORKSPACE],
            WORKSPACE_KEY
        );

        let work = store.create_workspace("Work", Ink::Moss).unwrap();
        let keys = store.workspace_sync_ids().unwrap();
        let created = pushed(&store)
            .into_iter()
            .find(|m| m["data"]["name"] == "Work")
            .unwrap();
        assert_eq!(
            created["key"], keys[&work],
            "settings must address the same workspace as sync"
        );
        assert_ne!(keys[&work], work.0.to_string());

        store.delete_workspace(work).unwrap();
        assert!(!store.workspace_sync_ids().unwrap().contains_key(&work));
        store.unlink().unwrap();
        assert!(store.workspace_sync_ids().unwrap().is_empty());
    }
}
