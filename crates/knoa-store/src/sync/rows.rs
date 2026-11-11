//! Local rows in the protocol's shape: what a `put` of the whole row would carry.
//!
//! Only rows that have a sync id, under a parent that has one, are read; device-local
//! columns are never part of them.

use std::collections::BTreeMap;

use knoa_core::{
    Fields, Origin,
    protocol::{BookmarkPatch, BookmarkStatus, FolderPatch, ListedTag, WorkspacePatch},
};
use rusqlite::{Connection, OptionalExtension as _, params};

use knoa_core::WorkspaceId;

use crate::{Result, tag_list};

/// A synced row: its key, and every column it sends.
pub(crate) struct Synced<P> {
    pub key: String,
    pub row: P,
}

/// Whether a bookmark has a tag or folder, or the user removed it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Link {
    On {
        origin: Origin,
        probability: Option<f64>,
    },
    Excluded,
    Gone,
}

pub(crate) fn workspace(conn: &Connection, id: i64) -> Result<Option<Synced<WorkspacePatch>>> {
    let row = conn
        .query_row(
            "SELECT sync_id, name, color, position, instructions, screening, guideline,
                    only_listed_tags
             FROM workspaces WHERE id = ?1 AND sync_id IS NOT NULL",
            [id],
            |row| {
                let key: String = row.get(0)?;
                Ok(Synced {
                    row: WorkspacePatch {
                        id: Some(key.clone()),
                        name: Some(row.get(1)?),
                        color: Some(row.get(2)?),
                        position: Some(row.get(3)?),
                        instructions: Some(row.get(4)?),
                        screening: Some(row.get(5)?),
                        guideline: Some(row.get(6)?),
                        only_listed_tags: Some(row.get(7)?),
                        tag_list: None,
                    },
                    key,
                })
            },
        )
        .optional()?;
    let Some(mut synced) = row else {
        return Ok(None);
    };
    synced.row.tag_list = Some(listed_tags(conn, id)?);
    Ok(Some(synced))
}

fn listed_tags(conn: &Connection, workspace: i64) -> Result<Vec<ListedTag>> {
    Ok(tag_list::load(conn, WorkspaceId(workspace))?
        .into_iter()
        .map(|tag| ListedTag {
            tag: tag.name,
            description: tag.description,
        })
        .collect())
}

pub(crate) fn folder(conn: &Connection, id: i64) -> Result<Option<Synced<FolderPatch>>> {
    Ok(conn
        .query_row(
            "SELECT f.sync_id, w.sync_id, f.name, f.description, f.auto_assign, f.color, f.hue,
                    f.icon, f.position
             FROM folders f JOIN workspaces w ON w.id = f.workspace_id
             WHERE f.id = ?1 AND f.sync_id IS NOT NULL AND w.sync_id IS NOT NULL",
            [id],
            |row| {
                let key: String = row.get(0)?;
                Ok(Synced {
                    row: FolderPatch {
                        id: Some(key.clone()),
                        workspace_id: Some(row.get(1)?),
                        name: Some(row.get(2)?),
                        description: Some(row.get(3)?),
                        auto_assign: Some(row.get(4)?),
                        color: Some(row.get(5)?),
                        hue: Some(row.get(6)?),
                        icon: Some(row.get(7)?),
                        position: Some(row.get(8)?),
                    },
                    key,
                })
            },
        )
        .optional()?)
}

/// The bookmark's synced columns. The image columns are left out: the server sets them
/// when a device uploads the thumbnail.
pub(crate) fn bookmark(conn: &Connection, id: i64) -> Result<Option<Synced<BookmarkPatch>>> {
    let raw = conn
        .query_row(
            "SELECT b.sync_id, w.sync_id, b.url, b.title, b.site, b.added_at, b.up_next, b.kept,
                    b.deleted_at, b.browser_folder, b.status, b.kind, b.summary, b.fields,
                    b.extras, b.set_aside, b.model, b.organized_at, b.error
             FROM bookmarks b JOIN workspaces w ON w.id = b.workspace_id
             WHERE b.id = ?1 AND b.sync_id IS NOT NULL AND w.sync_id IS NOT NULL",
            [id],
            |row| {
                let key: String = row.get(0)?;
                let status: String = row.get(10)?;
                let json: (String, String) = (row.get(13)?, row.get(14)?);
                let row = BookmarkPatch {
                    id: Some(key.clone()),
                    workspace_id: Some(row.get(1)?),
                    url: Some(row.get(2)?),
                    title: Some(row.get(3)?),
                    site: Some(row.get(4)?),
                    added_at: Some(row.get(5)?),
                    up_next: Some(row.get(6)?),
                    kept: Some(row.get(7)?),
                    deleted_at: Some(row.get(8)?),
                    browser_folder: Some(row.get(9)?),
                    status: remote_status(&status),
                    kind: Some(row.get(11)?),
                    summary: Some(row.get(12)?),
                    set_aside: Some(row.get(15)?),
                    model: Some(row.get(16)?),
                    organized_at: Some(row.get(17)?),
                    error: Some(row.get(18)?),
                    ..BookmarkPatch::default()
                };
                Ok((key, row, json))
            },
        )
        .optional()?;
    let Some((key, mut row, (fields, extras))) = raw else {
        return Ok(None);
    };
    row.fields = Some(serde_json::from_str::<Fields>(&fields)?);
    row.extras = Some(serde_json::from_str(&extras)?);
    Ok(Some(Synced { key, row }))
}

/// The protocol's status for a local one. Queued and organizing belong to this Mac's
/// queue, so they send nothing.
fn remote_status(local: &str) -> Option<BookmarkStatus> {
    match local {
        "new" => Some(BookmarkStatus::New),
        "organized" => Some(BookmarkStatus::Organized),
        "failed" => Some(BookmarkStatus::Failed),
        _ => None,
    }
}

/// The bookmark's tags with their origins.
pub(crate) fn tags(conn: &Connection, bookmark: i64) -> Result<BTreeMap<String, Origin>> {
    let mut stmt =
        conn.prepare_cached("SELECT tag, origin FROM bookmark_tags WHERE bookmark_id = ?1")?;
    let rows = stmt.query_map([bookmark], |row| {
        Ok((row.get(0)?, Origin::from_slug(&row.get::<_, String>(1)?)))
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub(crate) fn tag(conn: &Connection, bookmark: i64, tag: &str) -> Result<Link> {
    let origin: Option<String> = conn
        .query_row(
            "SELECT origin FROM bookmark_tags WHERE bookmark_id = ?1 AND tag = ?2",
            params![bookmark, tag],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(origin) = origin {
        return Ok(Link::On {
            origin: Origin::from_slug(&origin),
            probability: None,
        });
    }
    let excluded = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM tag_exclusions WHERE bookmark_id = ?1 AND tag = ?2)",
        params![bookmark, tag],
        |row| row.get(0),
    )?;
    Ok(if excluded { Link::Excluded } else { Link::Gone })
}

pub(crate) fn folder_link(conn: &Connection, bookmark: i64, folder: i64) -> Result<Link> {
    let link: Option<(String, Option<f64>)> = conn
        .query_row(
            "SELECT origin, probability FROM bookmark_folders WHERE bookmark_id = ?1 AND folder_id = ?2",
            params![bookmark, folder],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    if let Some((origin, probability)) = link {
        return Ok(Link::On {
            origin: Origin::from_slug(&origin),
            probability,
        });
    }
    let excluded = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM folder_exclusions WHERE bookmark_id = ?1 AND folder_id = ?2)",
        params![bookmark, folder],
        |row| row.get(0),
    )?;
    Ok(if excluded { Link::Excluded } else { Link::Gone })
}

pub(crate) fn page_text(conn: &Connection, bookmark: i64) -> Result<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT page_text FROM bookmarks WHERE id = ?1",
            [bookmark],
            |row| row.get(0),
        )
        .optional()?
        .flatten())
}
