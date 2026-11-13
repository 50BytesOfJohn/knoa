//! The sync ids the recorder sends rows under, minted on first use.

use knoa_core::protocol::{bookmark_id, new_id};
use rusqlite::OptionalExtension as _;

use super::{
    super::{
        delivery,
        ids::{self, Table},
    },
    Recorder,
};
use crate::{Error, Result};

impl Recorder<'_> {
    /// The workspace's sync id, minted and sent whole if it had none. `None` when there is
    /// no such workspace.
    pub(super) fn workspace_key(&self, id: i64) -> Result<Option<String>> {
        if !self.linked {
            return Ok(None);
        }
        if let Some(key) = ids::sync_id(self.conn, Table::Workspaces, id)? {
            return Ok(Some(key));
        }
        let key = new_id();
        if !ids::set_sync_id(self.conn, Table::Workspaces, id, &key)? {
            return Ok(None);
        }
        self.put_whole_workspace(id)?;
        Ok(Some(key))
    }

    pub(super) fn folder_key(&self, id: i64) -> Result<Option<String>> {
        if !self.linked {
            return Ok(None);
        }
        if let Some(key) = ids::sync_id(self.conn, Table::Folders, id)? {
            return Ok(Some(key));
        }
        let Some(workspace) = self.parent(Table::Folders, id)? else {
            return Ok(None);
        };
        self.workspace_key(workspace)?;
        let key = new_id();
        ids::set_sync_id(self.conn, Table::Folders, id, &key)?;
        self.put_whole_folder(id)?;
        Ok(Some(key))
    }

    /// Blocked bookmarks retain their changes locally until a full snapshot can retry.
    pub(super) fn bookmark_key(&self, id: i64) -> Result<Option<String>> {
        if !self.linked || delivery::is_blocked(self.conn, id)? {
            return Ok(None);
        }
        if let Some(key) = ids::sync_id(self.conn, Table::Bookmarks, id)? {
            return Ok(Some(key));
        }
        let Some(workspace) = self.parent(Table::Bookmarks, id)? else {
            return Ok(None);
        };
        let Some(namespace) = self.workspace_key(workspace)? else {
            return Ok(None);
        };
        let url: String =
            self.conn
                .query_row("SELECT url FROM bookmarks WHERE id = ?1", [id], |row| {
                    row.get(0)
                })?;
        let key = bookmark_id(&namespace, &url).ok_or(Error::InvalidSyncId(namespace))?;
        ids::set_sync_id(self.conn, Table::Bookmarks, id, &key)?;
        self.put_whole_bookmark(id)?;
        Ok(Some(key))
    }

    /// The workspace a folder or bookmark belongs to.
    fn parent(&self, table: Table, id: i64) -> Result<Option<i64>> {
        let sql = match table {
            Table::Folders => "SELECT workspace_id FROM folders WHERE id = ?1",
            Table::Bookmarks => "SELECT workspace_id FROM bookmarks WHERE id = ?1",
            Table::Workspaces => return Ok(None),
        };
        Ok(self
            .conn
            .query_row(sql, [id], |row| row.get(0))
            .optional()?)
    }
}
