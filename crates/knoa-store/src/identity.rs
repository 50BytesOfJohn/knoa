//! Identity checks for results arriving after a bookmark may have been replaced.

use knoa_core::{Bookmark, BookmarkId, WorkspaceId};
use rusqlite::{Connection, OptionalExtension as _, params};

use crate::{Result, Store};

/// Captured before background work so a replacement row cannot receive its result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookmarkIdentity {
    id: BookmarkId,
    workspace: WorkspaceId,
    url: String,
    sync_id: Option<String>,
}

impl Store {
    /// Captures identity only if the displayed bookmark still owns its row.
    pub fn bookmark_identity(&self, bookmark: &Bookmark) -> Result<Option<BookmarkIdentity>> {
        Ok(self
            .conn
            .query_row(
                "SELECT sync_id FROM bookmarks WHERE id = ?1 AND workspace_id = ?2 AND url = ?3",
                params![bookmark.id.0, bookmark.workspace.0, bookmark.url],
                |row| {
                    Ok(BookmarkIdentity {
                        id: bookmark.id,
                        workspace: bookmark.workspace,
                        url: bookmark.url.clone(),
                        sync_id: row.get(0)?,
                    })
                },
            )
            .optional()?)
    }
}

impl BookmarkIdentity {
    pub(crate) fn is_current(&self, conn: &Connection) -> Result<bool> {
        // An unlinked bookmark can acquire a sync id while its request is in flight.
        Ok(conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM bookmarks
             WHERE id = ?1 AND workspace_id = ?2 AND url = ?3
               AND (?4 IS NULL OR sync_id = ?4))",
            params![self.id.0, self.workspace.0, self.url, self.sync_id],
            |row| row.get(0),
        )?)
    }

    pub(crate) fn id(&self) -> BookmarkId {
        self.id
    }
}
