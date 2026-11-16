//! Pulled folders.

use knoa_core::protocol::{Entity, Folder, FolderPatch};
use rusqlite::{Connection, params};

use super::{Outcome, columns::Columns, rebased, release_orphans};
use crate::{
    Result,
    sync::{
        ids::{self, Table},
        outbox,
    },
};

pub(super) fn put(conn: &Connection, key: &str, row: Folder) -> Result<Outcome> {
    let Some(workspace) = ids::local_id(conn, Table::Workspaces, &row.workspace_id)? else {
        return Ok(Outcome::WaitsForParent);
    };
    let pending = outbox::pending_for(conn, Entity::Folder, key)?;
    if pending.deleted {
        return Ok(Outcome::Done);
    }
    let patch = rebased(FolderPatch::from(row), &pending)?;
    let parent = match patch.parent_id.clone().flatten() {
        Some(parent) => match ids::local_id(conn, Table::Folders, &parent)? {
            Some(local) => Some(local),
            None => return Ok(Outcome::WaitsForParent),
        },
        None => None,
    };
    let id = if let Some(id) = ids::local_id(conn, Table::Folders, key)? {
        id
    } else {
        conn.execute(
            "INSERT INTO folders (workspace_id, name, position, sync_id) VALUES (?1, '', 0, ?2)",
            params![workspace, key],
        )?;
        conn.last_insert_rowid()
    };
    let mut columns = Columns::default();
    columns.add("workspace_id", patch.workspace_id.map(|_| workspace));
    columns.add("parent_id", patch.parent_id.map(|_| parent));
    columns.add("name", patch.name);
    columns.add("description", patch.description);
    columns.add("auto_assign", patch.auto_assign);
    columns.add("color", patch.color);
    columns.add("hue", patch.hue);
    columns.add("icon", patch.icon);
    columns.add("position", patch.position);
    columns.update(conn, "folders", id)?;
    Ok(Outcome::Done)
}

/// Subfolders move up to the deleted folder's parent, as the server moves them: an older
/// app can delete a folder without knowing it has any.
pub(super) fn delete(conn: &Connection, key: &str) -> Result<Outcome> {
    if let Some(id) = ids::local_id(conn, Table::Folders, key)? {
        conn.execute(
            "UPDATE folders SET parent_id = (SELECT parent_id FROM folders WHERE id = ?1)
             WHERE parent_id = ?1",
            [id],
        )?;
        conn.execute("DELETE FROM folders WHERE id = ?1", [id])?;
        outbox::forget(conn, Entity::Folder, key)?;
    }
    release_orphans(
        conn,
        "entity = 'bookmark_folder' AND json_extract(change, '$.data.folder_id') = ?1",
        key,
    )
    .map(|()| Outcome::Done)
}
