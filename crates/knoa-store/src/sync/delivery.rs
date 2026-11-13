//! Persisted evidence of bookmark publication and reasons a creation waits locally.

use rusqlite::{
    Connection, OptionalExtension as _, params,
    types::{FromSql, FromSqlError, FromSqlResult, ValueRef},
};

use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BookmarkDelivery {
    PendingCreate,
    Blocked(BlockReason),
    Confirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BlockReason {
    Quota,
    Invalid,
}

impl BookmarkDelivery {
    fn slug(self) -> &'static str {
        match self {
            Self::PendingCreate => "pending_create",
            Self::Blocked(BlockReason::Quota) => "blocked_quota",
            Self::Blocked(BlockReason::Invalid) => "blocked_invalid",
            Self::Confirmed => "confirmed",
        }
    }
}

impl FromSql for BookmarkDelivery {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_str()? {
            "pending_create" => Ok(Self::PendingCreate),
            "blocked_quota" => Ok(Self::Blocked(BlockReason::Quota)),
            "blocked_invalid" => Ok(Self::Blocked(BlockReason::Invalid)),
            "confirmed" => Ok(Self::Confirmed),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

pub(super) fn state(conn: &Connection, bookmark: i64) -> Result<Option<BookmarkDelivery>> {
    Ok(conn
        .query_row(
            "SELECT delivery FROM bookmarks WHERE id = ?1",
            [bookmark],
            |row| row.get(0),
        )
        .optional()?)
}

pub(super) fn set(conn: &Connection, bookmark: i64, state: BookmarkDelivery) -> Result<()> {
    conn.execute(
        "UPDATE bookmarks SET delivery = ?2 WHERE id = ?1",
        params![bookmark, state.slug()],
    )?;
    Ok(())
}

pub(super) fn is_blocked(conn: &Connection, bookmark: i64) -> Result<bool> {
    Ok(matches!(
        state(conn, bookmark)?,
        Some(BookmarkDelivery::Blocked(_))
    ))
}

/// A patch or delete cannot prove a creation succeeded. Only the full put can.
pub(super) fn confirm_creation(conn: &Connection, mid: &str) -> Result<()> {
    conn.execute(
        "UPDATE bookmarks SET delivery = 'confirmed'
         WHERE delivery = 'pending_create' AND sync_id = (
             SELECT key FROM outbox WHERE mid = ?1 AND entity = 'bookmark' AND op = 'put'
                 AND json_type(data, '$.url') = 'text'
                 AND json_type(data, '$.workspace_id') = 'text')",
        [mid],
    )?;
    Ok(())
}
