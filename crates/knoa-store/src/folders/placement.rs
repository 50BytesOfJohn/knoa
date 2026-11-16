//! Where a folder sits: its parent, and its place among its siblings.

use knoa_core::FolderId;
use rusqlite::{Connection, params};

use super::tree;
use crate::{Result, Store, sync::Recorder};

impl Store {
    /// Moves the folder, with everything under it, last among `parent`'s children, or to
    /// the top level for `None`.
    ///
    /// # Errors
    ///
    /// [`crate::Error::FolderPlacement`] when the parent is in another workspace, inside the
    /// folder, or the subtree would be too deep.
    pub fn move_folder(&mut self, id: FolderId, parent: Option<FolderId>) -> Result<()> {
        let tx = self.conn.transaction()?;
        let recorder = Recorder::user(&tx)?;
        let before = recorder.folder_before(id.0)?;
        place(&tx, id, parent)?;
        recorder.folder_changed(id.0, before)?;
        tx.commit()?;
        Ok(())
    }
}

/// Sets the folder's parent and puts it last among its new siblings. Not recorded: the
/// caller wraps it in the recorder's before and after.
pub(super) fn place(conn: &Connection, id: FolderId, parent: Option<FolderId>) -> Result<()> {
    let Some(tree) = tree::load_around(conn, id)? else {
        return Ok(());
    };
    if tree.get(id).map(|f| f.parent) == Some(parent) {
        return Ok(());
    }
    tree.check_move(id, parent)?;
    let workspace: i64 = conn.query_row(
        "SELECT workspace_id FROM folders WHERE id = ?1",
        [id.0],
        |row| row.get(0),
    )?;
    conn.execute(
        "UPDATE folders SET parent_id = ?2, position = ?3 WHERE id = ?1",
        params![
            id.0,
            parent.map(|p| p.0),
            next_position(conn, workspace, parent)?
        ],
    )?;
    Ok(())
}

/// One past the last position among `parent`'s children in the workspace.
pub(super) fn next_position(
    conn: &Connection,
    workspace: i64,
    parent: Option<FolderId>,
) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT coalesce(max(position), 0) + 1 FROM folders
         WHERE workspace_id = ?1 AND parent_id IS ?2",
        params![workspace, parent.map(|p| p.0)],
        |row| row.get(0),
    )?)
}

#[cfg(test)]
mod tests {
    use knoa_core::{Ink, MoveError, NewFolder};

    use crate::{Error, Store, test_support::WORKSPACE};

    #[test]
    fn a_folder_only_moves_under_a_folder_of_its_own_workspace() {
        let mut store = Store::open_in_memory().unwrap();
        let work = store.create_workspace("Work", Ink::Moss).unwrap();
        let reading = store
            .create_folder(WORKSPACE, &NewFolder::named("Reading"))
            .unwrap();
        let elsewhere = store
            .create_folder(work, &NewFolder::named("Docs"))
            .unwrap();

        let moved = store.move_folder(reading, Some(elsewhere));

        assert!(matches!(
            moved,
            Err(Error::FolderPlacement(MoveError::UnknownParent))
        ));
        assert_eq!(store.folders(WORKSPACE).unwrap().folders()[0].parent, None);
    }
}
