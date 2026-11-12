//! Pulled workspaces. Which browsers feed one and where its links open stay this Mac's.

use knoa_core::{
    ListedTag, WorkspaceId,
    protocol::{Entity, Workspace, WorkspacePatch},
};
use rusqlite::Connection;

use super::{Outcome, columns::Columns, rebased, release_orphans};
use crate::{
    Result,
    sync::{
        ids::{self, Table},
        outbox,
    },
    tag_list, workspaces,
};

pub(super) fn put(conn: &Connection, key: &str, row: Workspace) -> Result<Outcome> {
    let pending = outbox::pending_for(conn, Entity::Workspace, key)?;
    if pending.deleted {
        return Ok(Outcome::Done);
    }
    let id = if let Some(id) = ids::local_id(conn, Table::Workspaces, key)? {
        id
    } else {
        conn.execute(
            "INSERT INTO workspaces (name, position, sync_id) VALUES ('', 0, ?1)",
            [key],
        )?;
        conn.last_insert_rowid()
    };
    let patch = rebased(WorkspacePatch::from(row), &pending)?;
    let mut columns = Columns::default();
    columns.add("name", patch.name);
    columns.add("color", patch.color);
    columns.add("position", patch.position);
    columns.add("instructions", patch.instructions);
    columns.add("screening", patch.screening);
    columns.add("guideline", patch.guideline);
    columns.add("only_listed_tags", patch.only_listed_tags);
    columns.update(conn, "workspaces", id)?;
    if let Some(tags) = patch.tag_list {
        let tags: Vec<ListedTag> = tags
            .into_iter()
            .map(|tag| ListedTag {
                name: tag.tag,
                description: tag.description,
            })
            .collect();
        tag_list::replace(conn, WorkspaceId(id), &tags)?;
    }
    Ok(Outcome::Done)
}

/// The app always needs a workspace, so the last one's deletion waits in the inbox until
/// another arrives or is created here.
pub(super) fn delete(conn: &Connection, key: &str) -> Result<Outcome> {
    let Some(id) = ids::local_id(conn, Table::Workspaces, key)? else {
        return Ok(Outcome::Done);
    };
    let count: usize = conn.query_row("SELECT count(*) FROM workspaces", [], |row| row.get(0))?;
    if count <= 1 {
        return Ok(Outcome::WaitsForParent);
    }
    workspaces::delete(conn, id)?;
    outbox::forget(conn, Entity::Workspace, key)?;
    release_orphans(
        conn,
        "entity IN ('folder', 'bookmark') AND json_extract(change, '$.data.workspace_id') = ?1",
        key,
    )
    .map(|()| Outcome::Done)
}
