//! Knoa's library on disk: one SQLite file with workspaces, bookmarks, folders, tags and
//! tag lists, the organizing queue, AI usage totals and a full-text index.
//!
//! `Store` is synchronous. The app runs it on GPUI's background executor, never on the UI
//! thread, with one read-write connection and one read-only connection to the same file.

mod folders;
mod import;
mod jobs;
mod read;
mod schema;
mod search;
mod tag_list;
mod tags;
mod trash;
mod usage;
mod workspaces;

use std::{path::Path, time::Duration};

use rusqlite::{Connection, OpenFlags};

pub use import::{AddedLink, ImportReport, NewBookmarks};
pub use jobs::Job;
pub use read::Counts;

// The wrapped errors are in the message and deliberately not the `source()`: an error chain
// shown as `{:#}` would print them twice, and rusqlite's own source would add a third copy.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database error: {0}")]
    Sqlite(rusqlite::Error),
    #[error("stored JSON is invalid: {0}")]
    Json(serde_json::Error),
    #[error(
        "the library was saved by a newer version of Knoa (schema {found}, this version knows up to {supported}); update Knoa to open it"
    )]
    NewerDatabase { found: usize, supported: usize },
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Error {
        Error::Sqlite(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Json(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// In WAL mode the writer and a reader rarely collide, but a checkpoint or WAL recovery can
/// still lock out the other connection for a moment.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

pub struct Store {
    conn: Connection,
}

impl Store {
    /// Opens or creates the database and brings its schema up to date.
    pub fn open(path: &Path) -> Result<Store> {
        Store::init(Connection::open(path)?)
    }

    /// Opens an existing database for reading only; every write through it fails. It
    /// doesn't migrate, so open it after `Store::open` has brought the schema up to date.
    pub fn open_read_only(path: &Path) -> Result<Store> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        conn.busy_timeout(BUSY_TIMEOUT)?;
        Ok(Store { conn })
    }

    pub fn open_in_memory() -> Result<Store> {
        Store::init(Connection::open_in_memory()?)
    }

    /// Runs `read` against one snapshot of the database, so several queries agree with each
    /// other even while another connection writes.
    pub fn snapshot<T>(&self, read: impl FnOnce(&Store) -> Result<T>) -> Result<T> {
        // Nothing is written, so the rollback when `_snapshot` drops ends it as well as a
        // commit would.
        let _snapshot = self.conn.unchecked_transaction()?;
        read(self)
    }

    fn init(conn: Connection) -> Result<Store> {
        conn.busy_timeout(BUSY_TIMEOUT)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        let mut store = Store { conn };
        schema::migrate(&mut store.conn)?;
        Ok(store)
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use knoa_core::{NewBookmark, WorkspaceId};

    /// The workspace every database starts with.
    pub const WORKSPACE: WorkspaceId = WorkspaceId(1);

    pub fn bookmark(url: &str, title: &str, added_at: i64) -> NewBookmark {
        NewBookmark {
            url: url.into(),
            title: title.into(),
            added_at,
            browser_guid: None,
            browser_folder: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reader_sees_committed_writes_and_cannot_write() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("knoa.db");
        let mut writer = Store::open(&path).unwrap();
        let mut reader = Store::open_read_only(&path).unwrap();

        let id = writer.create_workspace("Work", 2).unwrap();
        let seen = reader.workspaces().unwrap();
        assert!(seen.iter().any(|w| w.id == id && w.name == "Work"));

        assert!(reader.create_workspace("Sneaky", 0).is_err());
        assert_eq!(writer.workspaces().unwrap().len(), seen.len());
    }
}
