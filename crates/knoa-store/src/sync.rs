//! Cloud sync's local side: sync ids, the outbox of local changes, linking the library to
//! an account, and applying what the server sends.
//!
//! Until the library is linked nothing here does anything: no row has a sync id and no
//! change is recorded, so Local mode is the app as it always was.

mod apply;
mod device;
mod ids;
mod link;
mod outbox;
mod recorder;
mod rows;
mod unsent;

use knoa_core::protocol::{MAX_PUSH, MAX_PUSH_BYTES, Mutation};
use rusqlite::{Connection, OptionalExtension as _};

pub(crate) use apply::apply_held;
pub use device::{CloudJob, ThumbnailDownload, ThumbnailUpload};
pub use link::AccountRows;
pub(crate) use recorder::Recorder;
pub(crate) use rows::page_text;
pub use unsent::Settled;

use crate::{Result, Store};

impl Store {
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

    /// Drops pushed mutations. The server's answer for each is final, whether it applied,
    /// ignored or rejected it.
    pub fn acknowledge(&mut self, mids: &[String]) -> Result<()> {
        let tx = self.conn.transaction()?;
        outbox::acknowledge(&tx, mids)?;
        tx.commit()?;
        Ok(())
    }
}

pub(crate) fn linked(conn: &Connection) -> Result<bool> {
    Ok(
        conn.query_row("SELECT EXISTS (SELECT 1 FROM sync_state)", [], |row| {
            row.get(0)
        })?,
    )
}
