//! Pulled tags, folder links and page text. Locally a removed tag or folder is a row in
//! the exclusion tables, not a link with `excluded` set.
//!
//! These rows are always pushed whole, so any pending local change to one wins outright.

use knoa_core::protocol::{BookmarkFolder, BookmarkTag, Entity};
use rusqlite::{Connection, params};

use super::Outcome;
use crate::{
    Result, search,
    sync::{
        ids::{self, Table},
        outbox,
    },
};

pub(super) fn put_tag(conn: &Connection, key: &str, row: &BookmarkTag) -> Result<Outcome> {
    let Some(bookmark) = ids::local_id(conn, Table::Bookmarks, &row.bookmark_id)? else {
        return Ok(Outcome::WaitsForParent);
    };
    if has_pending(conn, Entity::BookmarkTag, key)? {
        return Ok(Outcome::Done);
    }
    let pair = params![bookmark, row.tag];
    if row.excluded {
        conn.execute(
            "DELETE FROM bookmark_tags WHERE bookmark_id = ?1 AND tag = ?2",
            pair,
        )?;
        conn.execute("INSERT OR IGNORE INTO tag_exclusions VALUES (?1, ?2)", pair)?;
    } else {
        conn.execute(
            "DELETE FROM tag_exclusions WHERE bookmark_id = ?1 AND tag = ?2",
            pair,
        )?;
        conn.execute(
            "INSERT INTO bookmark_tags VALUES (?1, ?2, ?3) ON CONFLICT DO UPDATE SET origin = ?3",
            params![bookmark, row.tag, row.origin.slug()],
        )?;
    }
    search::reindex(conn, bookmark)?;
    Ok(Outcome::Done)
}

pub(super) fn delete_tag(conn: &Connection, key: &str) -> Result<Outcome> {
    let Some((bookmark, tag)) = split(conn, key)? else {
        return Ok(Outcome::Done);
    };
    if has_pending(conn, Entity::BookmarkTag, key)? {
        return Ok(Outcome::Done);
    }
    let pair = params![bookmark, tag];
    conn.execute(
        "DELETE FROM bookmark_tags WHERE bookmark_id = ?1 AND tag = ?2",
        pair,
    )?;
    conn.execute(
        "DELETE FROM tag_exclusions WHERE bookmark_id = ?1 AND tag = ?2",
        pair,
    )?;
    search::reindex(conn, bookmark)?;
    Ok(Outcome::Done)
}

pub(super) fn put_folder_link(
    conn: &Connection,
    key: &str,
    row: &BookmarkFolder,
) -> Result<Outcome> {
    let bookmark = ids::local_id(conn, Table::Bookmarks, &row.bookmark_id)?;
    let folder = ids::local_id(conn, Table::Folders, &row.folder_id)?;
    let (Some(bookmark), Some(folder)) = (bookmark, folder) else {
        return Ok(Outcome::WaitsForParent);
    };
    if has_pending(conn, Entity::BookmarkFolder, key)? {
        return Ok(Outcome::Done);
    }
    let pair = params![bookmark, folder];
    if row.excluded {
        conn.execute(
            "DELETE FROM bookmark_folders WHERE bookmark_id = ?1 AND folder_id = ?2",
            pair,
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO folder_exclusions VALUES (?1, ?2)",
            pair,
        )?;
    } else {
        conn.execute(
            "DELETE FROM folder_exclusions WHERE bookmark_id = ?1 AND folder_id = ?2",
            pair,
        )?;
        conn.execute(
            "INSERT INTO bookmark_folders (bookmark_id, folder_id, origin, probability)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT DO UPDATE SET origin = ?3, probability = ?4",
            params![bookmark, folder, row.origin.slug(), row.probability],
        )?;
    }
    Ok(Outcome::Done)
}

pub(super) fn delete_folder_link(conn: &Connection, key: &str) -> Result<Outcome> {
    let Some((bookmark, folder)) = split(conn, key)? else {
        return Ok(Outcome::Done);
    };
    let Some(folder) = ids::local_id(conn, Table::Folders, folder)? else {
        return Ok(Outcome::Done);
    };
    if has_pending(conn, Entity::BookmarkFolder, key)? {
        return Ok(Outcome::Done);
    }
    let pair = params![bookmark, folder];
    conn.execute(
        "DELETE FROM bookmark_folders WHERE bookmark_id = ?1 AND folder_id = ?2",
        pair,
    )?;
    conn.execute(
        "DELETE FROM folder_exclusions WHERE bookmark_id = ?1 AND folder_id = ?2",
        pair,
    )?;
    Ok(Outcome::Done)
}

pub(super) fn put_text(conn: &Connection, key: &str, text: &str) -> Result<Outcome> {
    set_text(conn, key, Some(text), Outcome::WaitsForParent)
}

pub(super) fn delete_text(conn: &Connection, key: &str) -> Result<Outcome> {
    set_text(conn, key, None, Outcome::Done)
}

fn set_text(
    conn: &Connection,
    key: &str,
    text: Option<&str>,
    without_bookmark: Outcome,
) -> Result<Outcome> {
    let Some(bookmark) = ids::local_id(conn, Table::Bookmarks, key)? else {
        return Ok(without_bookmark);
    };
    if has_pending(conn, Entity::BookmarkText, key)? {
        return Ok(Outcome::Done);
    }
    conn.execute(
        "UPDATE bookmarks SET page_text = ?2 WHERE id = ?1",
        params![bookmark, text],
    )?;
    search::reindex(conn, bookmark)?;
    Ok(Outcome::Done)
}

fn has_pending(conn: &Connection, entity: Entity, key: &str) -> Result<bool> {
    Ok(!outbox::pending_for(conn, entity, key)?.is_empty())
}

/// A link key's local bookmark and the rest of the key, when the bookmark is here.
fn split<'k>(conn: &Connection, key: &'k str) -> Result<Option<(i64, &'k str)>> {
    let Some((bookmark, rest)) = key.split_once('/') else {
        return Ok(None);
    };
    Ok(ids::local_id(conn, Table::Bookmarks, bookmark)?.map(|id| (id, rest)))
}
