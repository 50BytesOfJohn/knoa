//! Cloud sync's local side: sync ids, the outbox of local changes, linking the library to
//! an account, and applying what the server sends.
//!
//! Until the library is linked nothing here does anything: no row has a sync id and no
//! change is recorded, so Local mode is the app as it always was.

mod apply;
mod ids;
mod link;
mod outbox;
mod recorder;
mod rows;

use knoa_core::protocol::Mutation;
use rusqlite::{Connection, OptionalExtension as _};

pub(crate) use recorder::Recorder;

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

    /// The oldest `limit` local changes not pushed yet, in the order they were made.
    pub fn pending(&self, limit: usize) -> Result<Vec<Mutation>> {
        outbox::pending(&self.conn, limit)
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
