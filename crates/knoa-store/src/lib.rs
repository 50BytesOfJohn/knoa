//! Knoa's library on disk: one SQLite file with workspaces, bookmarks, folders, tags and
//! tag lists, the organizing queue, AI usage totals and a full-text index.
//!
//! `Store` is synchronous. The app runs it on GPUI's background executor, never on the UI
//! thread, with one read-write connection and one read-only connection to the same file.

mod add_link;
mod folders;
mod import;
mod jobs;
mod read;
mod schema;
mod search;
mod sync;
mod tag_list;
mod tags;
mod trash;
mod usage;
mod workspaces;

use std::{path::Path, time::Duration};

use rusqlite::{Connection, OpenFlags};

pub use add_link::AddedLink;
pub use import::{ImportReport, NewBookmarks};
pub use jobs::Job;
pub use read::Counts;
pub use sync::{CloudJob, ThumbnailDownload, ThumbnailUpload};

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
    #[error("the library is linked to another account ({linked}); unlink it first")]
    OtherAccount { linked: String },
    #[error("the library isn't linked to an account")]
    NotLinked,
    #[error("sync id {0:?} isn't a UUID")]
    InvalidSyncId(String),
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
    use knoa_core::{
        NewBookmark, WorkspaceId,
        protocol::{Change, Mutation},
    };
    use serde_json::{Value, json};

    use crate::Store;

    /// The workspace every database starts with.
    pub const WORKSPACE: WorkspaceId = WorkspaceId(1);

    /// The id the account knows `WORKSPACE` by, once linked.
    pub const WORKSPACE_KEY: &str = "6f1c2a3e-8b4d-4e5f-9a6b-7c8d9e0f1a2b";

    /// A store linked to an account whose only row is a workspace that `WORKSPACE` adopts.
    pub fn linked(store: &mut Store) {
        let personal = change(json!({
            "entity": "workspace", "key": WORKSPACE_KEY, "version": 1, "deleted": false,
            "data": workspace_row(WORKSPACE_KEY, "Personal"),
        }));
        store.link("user-1", &[personal], 1).unwrap();
    }

    pub fn change(json: Value) -> Change {
        serde_json::from_value(json).unwrap()
    }

    pub fn workspace_row(id: &str, name: &str) -> Value {
        json!({ "id": id, "name": name, "color": 0, "position": 1, "instructions": "",
                "screening": false, "guideline": "", "only_listed_tags": false, "tag_list": [] })
    }

    pub fn bookmark_row(id: &str, workspace: &str, url: &str, title: &str) -> Value {
        json!({ "id": id, "workspace_id": workspace, "url": url, "title": title, "site": "a.dev",
                "added_at": 5, "up_next": false, "kept": false, "deleted_at": null,
                "browser_folder": null, "status": "new", "kind": null, "summary": null,
                "fields": {}, "extras": [], "set_aside": null, "model": null,
                "organized_at": null, "error": null, "image_key": null, "image_width": null,
                "image_height": null })
    }

    /// Everything waiting to be pushed, as JSON for compact assertions.
    pub fn pushed(store: &Store) -> Vec<Value> {
        store
            .pending(1000)
            .unwrap()
            .iter()
            .map(|mutation: &Mutation| serde_json::to_value(mutation).unwrap())
            .collect()
    }

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
    use knoa_core::Ink;

    use super::*;

    #[test]
    fn reader_sees_committed_writes_and_cannot_write() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("knoa.db");
        let mut writer = Store::open(&path).unwrap();
        let mut reader = Store::open_read_only(&path).unwrap();

        let id = writer.create_workspace("Work", Ink::Moss).unwrap();
        let seen = reader.workspaces().unwrap();
        assert!(seen.iter().any(|w| w.id == id && w.name == "Work"));

        assert!(reader.create_workspace("Sneaky", Ink::Ochre).is_err());
        assert_eq!(writer.workspaces().unwrap().len(), seen.len());
    }
}
