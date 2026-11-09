//! Knoa's library on disk: one SQLite file with workspaces, bookmarks, folders, tags and
//! tag lists, the organizing queue, AI usage totals and a full-text index.
//!
//! `Store` is synchronous. The app runs it on GPUI's background executor, never on the UI
//! thread.

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

use std::path::Path;

use rusqlite::Connection;

pub use import::{AddedLink, ImportReport, NewBookmarks};
pub use jobs::Job;
pub use read::Counts;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("stored JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Store {
    conn: Connection,
}

impl Store {
    /// Opens or creates the database and brings its schema up to date.
    pub fn open(path: &Path) -> Result<Store> {
        Store::init(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Store> {
        Store::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Store> {
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
