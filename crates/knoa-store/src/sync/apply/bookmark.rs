//! Pulled bookmarks. This Mac keeps its own browser columns, page evidence, queue and
//! thumbnail cache.

use knoa_core::protocol::{Bookmark, BookmarkPatch, BookmarkStatus, Entity};
use rusqlite::{Connection, OptionalExtension as _, params};

use super::{Outcome, columns::Columns, rebased, release_orphans};
use crate::{
    Result, search,
    sync::{
        ids::{self, Table},
        outbox,
    },
    trash,
};

pub(super) fn put(conn: &Connection, key: &str, row: Bookmark) -> Result<Outcome> {
    let Some(workspace) = ids::local_id(conn, Table::Workspaces, &row.workspace_id)? else {
        return Ok(Outcome::WaitsForParent);
    };
    let pending = outbox::pending_for(conn, Entity::Bookmark, key)?;
    if pending.deleted {
        return Ok(Outcome::Done);
    }
    let id = local_row(conn, key, workspace, &row.url)?;
    let patch = rebased(BookmarkPatch::from(row), &pending)?;
    if matches!(patch.deleted_at, Some(Some(_))) {
        leave_queue(conn, id)?;
    }
    if let Some(status) = patch.status {
        set_status(conn, id, status)?;
    }
    if let Some(image_key) = &patch.image_key {
        set_image(conn, id, image_key.as_deref(), &patch)?;
    }
    columns(patch)?.update(conn, "bookmarks", id)?;
    search::reindex(conn, id)?;
    Ok(Outcome::Done)
}

/// A tombstone wins over local edits, unless this Mac made the bookmark again since: its
/// pending `put` sends the whole row, and the server brings it back.
pub(super) fn delete(conn: &Connection, key: &str) -> Result<Outcome> {
    if let Some(id) = ids::local_id(conn, Table::Bookmarks, key)? {
        let pending = outbox::pending_for(conn, Entity::Bookmark, key)?;
        if pending.columns.contains("url") {
            return Ok(Outcome::Done);
        }
        trash::delete_for_good(conn, "id = ?1", id)?;
        outbox::forget(conn, Entity::Bookmark, key)?;
    }
    release_orphans(
        conn,
        "entity IN ('bookmark_tag', 'bookmark_folder', 'bookmark_text')
         AND json_extract(change, '$.data.bookmark_id') = ?1",
        key,
    )
    .map(|()| Outcome::Done)
}

/// The local row for the pulled one: by sync id, else the workspace's bookmark for the
/// same URL, which takes the server's id, else a new row.
fn local_row(conn: &Connection, key: &str, workspace: i64, url: &str) -> Result<i64> {
    if let Some(id) = ids::local_id(conn, Table::Bookmarks, key)? {
        return Ok(id);
    }
    let same_url: Option<i64> = conn
        .query_row(
            "SELECT id FROM bookmarks WHERE workspace_id = ?1 AND url = ?2",
            params![workspace, url],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(id) = same_url {
        ids::set_sync_id(conn, Table::Bookmarks, id, key)?;
        return Ok(id);
    }
    conn.execute(
        "INSERT INTO bookmarks (workspace_id, url, title, site, added_at, sync_id)
         VALUES (?1, ?2, '', '', 0, ?3)",
        params![workspace, url, key],
    )?;
    Ok(conn.last_insert_rowid())
}

fn columns(patch: BookmarkPatch) -> Result<Columns> {
    let mut columns = Columns::default();
    columns.add("title", patch.title);
    columns.add("site", patch.site);
    columns.add("added_at", patch.added_at);
    columns.add("up_next", patch.up_next);
    columns.add("kept", patch.kept);
    columns.add("deleted_at", patch.deleted_at);
    columns.add("browser_folder", patch.browser_folder);
    columns.add("kind", patch.kind);
    columns.add("summary", patch.summary);
    columns.add(
        "fields",
        patch
            .fields
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?,
    );
    columns.add(
        "extras",
        patch
            .extras
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?,
    );
    columns.add("set_aside", patch.set_aside);
    columns.add("model", patch.model);
    columns.add("organized_at", patch.organized_at);
    columns.add("error", patch.error);
    Ok(columns)
}

/// Trash takes a bookmark off this Mac's queue, as it does when trashed here.
fn leave_queue(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM jobs WHERE bookmark_id = ?1", [id])?;
    conn.execute(
        "UPDATE bookmarks SET status = 'new' WHERE id = ?1 AND status = 'queued'",
        [id],
    )?;
    Ok(())
}

/// While this Mac's queue has the bookmark, its status is the queue's.
fn set_status(conn: &Connection, id: i64, status: BookmarkStatus) -> Result<()> {
    let slug = match status {
        BookmarkStatus::New => "new",
        BookmarkStatus::Queued => "queued",
        BookmarkStatus::Organized => "organized",
        BookmarkStatus::Failed => "failed",
    };
    conn.execute(
        "UPDATE bookmarks SET status = ?2
         WHERE id = ?1 AND NOT EXISTS (SELECT 1 FROM jobs WHERE bookmark_id = ?1)",
        params![id, slug],
    )?;
    Ok(())
}

/// A cached thumbnail of an older server image is dropped, so it is fetched again. One
/// this Mac fetched itself and has not uploaded yet stays. The size describes the
/// cached file, so it changes only while there is none.
fn set_image(
    conn: &Connection,
    id: i64,
    image_key: Option<&str>,
    patch: &BookmarkPatch,
) -> Result<()> {
    conn.execute(
        "UPDATE bookmarks SET image_key = ?2, image_path = CASE
            WHEN pending_image IS NULL AND image_key IS NOT NULL AND image_key IS NOT ?2
            THEN NULL ELSE image_path END
         WHERE id = ?1",
        params![id, image_key],
    )?;
    conn.execute(
        "UPDATE bookmarks SET image_width = ?2, image_height = ?3
         WHERE id = ?1 AND image_path IS NULL",
        params![
            id,
            patch.image_width.flatten(),
            patch.image_height.flatten()
        ],
    )?;
    Ok(())
}
