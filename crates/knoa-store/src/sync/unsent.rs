//! Parking refused bookmark creations until room or a local correction permits a full retry.

use knoa_core::protocol::{Entity, MutationResult, MutationStatus, RejectReason};
use rusqlite::{Connection, params};

use super::{
    delivery::{self, BlockReason, BookmarkDelivery},
    ids::{self, Table},
    outbox, snapshot,
};
use crate::{Result, Store};

/// What the server's answers to a push left behind.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Settled {
    /// Bookmarks the plan's limit refused, now waiting on this Mac.
    pub parked: usize,
    /// Changes that broke the server's rules. Changes to a refused bookmark aren't
    /// counted: they go again with it.
    pub invalid: usize,
}

impl Store {
    /// Settles answered mutations, retaining refused creations until room becomes
    /// available or their synced content changes locally.
    pub fn settle(&mut self, results: &[MutationResult]) -> Result<Settled> {
        let tx = self.conn.transaction()?;
        let rows = results
            .iter()
            .map(|result| Ok((result, outbox::row_of(&tx, &result.mid)?)))
            .collect::<Result<Vec<_>>>()?;
        let mut settled = Settled::default();
        for (result, _) in &rows {
            if result.status == MutationStatus::Applied {
                delivery::confirm_creation(&tx, &result.mid)?;
            }
        }
        for (result, row) in &rows {
            if result.status != MutationStatus::Rejected {
                continue;
            }
            let reason = match result.reason {
                Some(RejectReason::Quota) => BlockReason::Quota,
                _ => BlockReason::Invalid,
            };
            if let Some((Entity::Bookmark, key)) = row
                && park(&tx, key, reason)?
            {
                match reason {
                    BlockReason::Quota => settled.parked += 1,
                    BlockReason::Invalid => settled.invalid += 1,
                }
            }
        }
        for (result, row) in &rows {
            let invalid = result.status == MutationStatus::Rejected
                && result.reason != Some(RejectReason::Quota);
            if invalid && !about_parked(&tx, row.as_ref())? {
                settled.invalid += 1;
            }
        }
        let mids: Vec<String> = results.iter().map(|result| result.mid.clone()).collect();
        outbox::acknowledge(&tx, &mids)?;
        tx.commit()?;
        Ok(settled)
    }

    /// How many bookmarks the plan's limit keeps on this Mac only.
    pub fn unsent_bookmarks(&self) -> Result<usize> {
        Ok(self.conn.query_row(
            "SELECT count(*) FROM bookmarks WHERE delivery = 'blocked_quota'",
            [],
            |row| row.get(0),
        )?)
    }

    /// Queues every bookmark the plan's limit refused again, whole. Returns how many.
    pub fn resend_unsent(&mut self) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let ids = {
            let mut stmt =
                tx.prepare("SELECT id FROM bookmarks WHERE delivery = 'blocked_quota'")?;
            let ids = stmt.query_map([], |row| row.get::<_, i64>(0))?;
            ids.collect::<rusqlite::Result<Vec<_>>>()?
        };
        for id in &ids {
            retry(&tx, *id)?;
        }
        tx.commit()?;
        Ok(ids.len())
    }
}

pub(super) fn retry(conn: &Connection, bookmark: i64) -> Result<()> {
    delivery::set(conn, bookmark, BookmarkDelivery::PendingCreate)?;
    snapshot::record_bookmark_snapshot(conn, bookmark)
}

/// Protects quota-blocked local values before a pulled row is applied.
pub(super) fn resume(conn: &Connection, bookmark: i64) -> Result<bool> {
    match delivery::state(conn, bookmark)? {
        Some(BookmarkDelivery::Blocked(BlockReason::Quota)) => {
            retry(conn, bookmark)?;
            Ok(true)
        }
        // Retrying invalid values would hide the server's row and be rejected again.
        _ => Ok(false),
    }
}

/// Keeps the bookmark known as `key` back, and forgets what the outbox still holds for
/// it, which the server would refuse too. False when there is no such bookmark.
fn park(conn: &Connection, key: &str, reason: BlockReason) -> Result<bool> {
    let Some(id) = ids::local_id(conn, Table::Bookmarks, key)? else {
        return Ok(false);
    };
    if delivery::state(conn, id)? != Some(BookmarkDelivery::PendingCreate) {
        return Ok(false);
    }
    delivery::set(conn, id, BookmarkDelivery::Blocked(reason))?;
    conn.execute(
        "DELETE FROM outbox
         WHERE (entity IN (?2, ?3) AND key = ?1)
            OR (entity IN (?4, ?5) AND substr(key, 1, length(?1) + 1) = ?1 || '/')",
        params![
            key,
            Entity::Bookmark.slug(),
            Entity::BookmarkText.slug(),
            Entity::BookmarkTag.slug(),
            Entity::BookmarkFolder.slug(),
        ],
    )?;
    Ok(true)
}

/// Whether an outbox row is about a parked bookmark.
fn about_parked(conn: &Connection, row: Option<&(Entity, String)>) -> Result<bool> {
    let Some((entity, key)) = row else {
        return Ok(false);
    };
    let bookmark = match entity {
        Entity::Bookmark | Entity::BookmarkText => key,
        Entity::BookmarkTag | Entity::BookmarkFolder => key.split('/').next().unwrap_or(key),
        Entity::Workspace | Entity::Folder => return Ok(false),
    };
    match ids::local_id(conn, Table::Bookmarks, bookmark)? {
        Some(id) => delivery::is_blocked(conn, id),
        None => Ok(false),
    }
}

#[cfg(test)]
mod tests;
