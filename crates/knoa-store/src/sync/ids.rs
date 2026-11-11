//! The ids the server knows rows by, kept beside the local integer ids.

use rusqlite::{Connection, OptionalExtension as _, params};

use crate::Result;

/// A table whose rows have a sync id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Table {
    Workspaces,
    Folders,
    Bookmarks,
}

impl Table {
    fn name(self) -> &'static str {
        match self {
            Table::Workspaces => "workspaces",
            Table::Folders => "folders",
            Table::Bookmarks => "bookmarks",
        }
    }
}

/// The row's sync id; `None` for a row that has none, or no such row.
pub(crate) fn sync_id(conn: &Connection, table: Table, id: i64) -> Result<Option<String>> {
    let sql = format!("SELECT sync_id FROM {} WHERE id = ?1", table.name());
    Ok(conn
        .query_row(&sql, [id], |row| row.get(0))
        .optional()?
        .flatten())
}

/// The local id of the row the server knows as `sync_id`.
pub(crate) fn local_id(conn: &Connection, table: Table, sync_id: &str) -> Result<Option<i64>> {
    let sql = format!("SELECT id FROM {} WHERE sync_id = ?1", table.name());
    Ok(conn
        .query_row(&sql, [sync_id], |row| row.get(0))
        .optional()?)
}

/// Gives the row its sync id. False when there is no such row.
pub(crate) fn set_sync_id(conn: &Connection, table: Table, id: i64, sync_id: &str) -> Result<bool> {
    let sql = format!("UPDATE {} SET sync_id = ?2 WHERE id = ?1", table.name());
    Ok(conn.execute(&sql, params![id, sync_id])? > 0)
}
