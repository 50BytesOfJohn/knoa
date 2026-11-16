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

    /// Moves the folder, with everything under it, under `parent` just before its child
    /// `before`, or last for `None` or a folder that isn't one of those children. The
    /// siblings are numbered again, and each one whose position changed is recorded.
    ///
    /// # Errors
    ///
    /// [`crate::Error::FolderPlacement`] when the parent is in another workspace, inside the
    /// folder, or the subtree would be too deep.
    pub fn place_folder(
        &mut self,
        id: FolderId,
        parent: Option<FolderId>,
        before: Option<FolderId>,
    ) -> Result<()> {
        let tx = self.conn.transaction()?;
        let Some(tree) = tree::load_around(&tx, id)? else {
            return Ok(());
        };
        tree.check_move(id, parent)?;
        let mut order: Vec<FolderId> = tree
            .children(parent)
            .map(|f| f.id)
            .filter(|&f| f != id)
            .collect();
        let at = before
            .and_then(|before| order.iter().position(|&f| f == before))
            .unwrap_or(order.len());
        order.insert(at, id);
        let recorder = Recorder::user(&tx)?;
        let befores = order
            .iter()
            .map(|f| recorder.folder_before(f.0))
            .collect::<Result<Vec<_>>>()?;
        tx.execute(
            "UPDATE folders SET parent_id = ?2 WHERE id = ?1",
            params![id.0, parent.map(|p| p.0)],
        )?;
        for (position, folder) in (1_i64..).zip(&order) {
            tx.execute(
                "UPDATE folders SET position = ?2 WHERE id = ?1",
                params![folder.0, position],
            )?;
        }
        for (folder, before) in order.iter().zip(befores) {
            recorder.folder_changed(folder.0, before)?;
        }
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

    use crate::{
        Error, Store,
        test_support::{WORKSPACE, applied, linked, pushed},
    };

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

    #[test]
    fn placing_a_folder_between_siblings_records_only_the_moved_positions() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let create = |store: &mut Store, name| {
            store
                .create_folder(WORKSPACE, &NewFolder::named(name))
                .unwrap()
        };
        let a = create(&mut store, "A");
        let b = create(&mut store, "B");
        let c = create(&mut store, "C");
        let child = store
            .create_folder(
                WORKSPACE,
                &NewFolder {
                    parent: Some(a),
                    ..NewFolder::named("Child")
                },
            )
            .unwrap();
        store.settle(&applied(&store)).unwrap();

        store.place_folder(c, None, Some(b)).unwrap();
        store.place_folder(child, None, Some(a)).unwrap();

        let names: Vec<String> = store
            .folders(WORKSPACE)
            .unwrap()
            .folders()
            .iter()
            .map(|f| f.name.clone())
            .collect();
        assert_eq!(names, ["Child", "A", "C", "B"]);
        let changes: Vec<_> = pushed(&store).iter().map(|m| m["data"].clone()).collect();
        assert!(
            changes
                .iter()
                .any(|data| data["parent_id"].is_null() && data.get("parent_id").is_some()),
            "the un-nested folder sends an explicit null parent: {changes:?}"
        );
        assert!(
            changes.iter().all(|data| data.get("name").is_none()),
            "only placement columns are sent: {changes:?}"
        );
    }
}
