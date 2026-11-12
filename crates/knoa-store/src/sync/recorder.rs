//! Turns local changes into outbox mutations, inside the change's own transaction.
//!
//! A store method makes its change as before and tells the recorder what it touched:
//! a row it created or is about to delete, a snapshot of a row before an update, or a
//! tag or folder link whose state may have changed. In Local mode every call is a no-op.

mod keys;
mod links;

use knoa_core::{
    Origin,
    protocol::{BookmarkPatch, Entity, FolderPatch, WorkspacePatch},
};
use rusqlite::Connection;
use serde::Serialize;
use serde_json::{Map, Value};

use super::{
    ids::{self, Table},
    linked, outbox,
    rows::{self, Synced},
    unsent,
};
use crate::Result;

pub(crate) struct Recorder<'c> {
    conn: &'c Connection,
    actor: Origin,
    linked: bool,
}

/// A row as it was before an update. Empty in Local mode, and for a row that had no sync
/// id yet, which is then sent whole.
pub(crate) struct Before<P>(Option<Synced<P>>);

impl<'c> Recorder<'c> {
    /// For changes the user made, including browser imports.
    pub fn user(conn: &'c Connection) -> Result<Recorder<'c>> {
        Recorder::new(conn, Origin::User)
    }

    /// For changes the organizer made.
    pub fn ai(conn: &'c Connection) -> Result<Recorder<'c>> {
        Recorder::new(conn, Origin::Ai)
    }

    pub fn new(conn: &'c Connection, actor: Origin) -> Result<Recorder<'c>> {
        Ok(Recorder {
            conn,
            actor,
            linked: linked(conn)?,
        })
    }

    pub fn is_linked(&self) -> bool {
        self.linked
    }

    pub fn created_workspace(&self, id: i64) -> Result<()> {
        self.workspace_key(id).map(drop)
    }

    pub fn created_folder(&self, id: i64) -> Result<()> {
        self.folder_key(id).map(drop)
    }

    pub fn created_bookmark(&self, id: i64) -> Result<()> {
        self.bookmark_key(id).map(drop)
    }

    pub fn workspace_before(&self, id: i64) -> Result<Before<WorkspacePatch>> {
        self.before(|| rows::workspace(self.conn, id))
    }

    pub fn workspace_changed(&self, id: i64, before: Before<WorkspacePatch>) -> Result<()> {
        let after = || rows::workspace(self.conn, id);
        self.changed(Entity::Workspace, before, after, || {
            self.created_workspace(id)
        })
    }

    pub fn folder_before(&self, id: i64) -> Result<Before<FolderPatch>> {
        self.before(|| rows::folder(self.conn, id))
    }

    pub fn folder_changed(&self, id: i64, before: Before<FolderPatch>) -> Result<()> {
        let after = || rows::folder(self.conn, id);
        self.changed(Entity::Folder, before, after, || self.created_folder(id))
    }

    pub fn bookmark_before(&self, id: i64) -> Result<Before<BookmarkPatch>> {
        if self.linked && unsent::is_unsent(self.conn, id)? {
            return Ok(Before(None));
        }
        self.before(|| rows::bookmark(self.conn, id))
    }

    pub fn bookmark_changed(&self, id: i64, before: Before<BookmarkPatch>) -> Result<()> {
        let after = || rows::bookmark(self.conn, id);
        self.changed(Entity::Bookmark, before, after, || {
            self.created_bookmark(id)
        })
    }

    /// Runs `change` and records the bookmark columns it changed.
    pub fn on_bookmark<T>(&self, id: i64, change: impl FnOnce() -> Result<T>) -> Result<T> {
        let before = self.bookmark_before(id)?;
        let result = change()?;
        self.bookmark_changed(id, before)?;
        Ok(result)
    }

    /// Call before deleting: the server deletes the workspace's folders and bookmarks too.
    pub fn deleting_workspace(&self, id: i64) -> Result<()> {
        self.deleting(Entity::Workspace, Table::Workspaces, id)
    }

    pub fn deleting_folder(&self, id: i64) -> Result<()> {
        self.deleting(Entity::Folder, Table::Folders, id)
    }

    pub fn deleting_bookmark(&self, id: i64) -> Result<()> {
        // The server never had it.
        if self.linked && unsent::is_unsent(self.conn, id)? {
            return Ok(());
        }
        self.deleting(Entity::Bookmark, Table::Bookmarks, id)
    }

    /// Sends the whole workspace, folder or bookmark.
    pub fn put_whole_workspace(&self, id: i64) -> Result<()> {
        self.put_whole(Entity::Workspace, rows::workspace(self.conn, id)?)
    }

    pub fn put_whole_folder(&self, id: i64) -> Result<()> {
        self.put_whole(Entity::Folder, rows::folder(self.conn, id)?)
    }

    pub fn put_whole_bookmark(&self, id: i64) -> Result<()> {
        self.put_whole(Entity::Bookmark, rows::bookmark(self.conn, id)?)
    }

    fn deleting(&self, entity: Entity, table: Table, id: i64) -> Result<()> {
        if !self.linked {
            return Ok(());
        }
        match ids::sync_id(self.conn, table, id)? {
            Some(key) => outbox::delete(self.conn, entity, &key, self.actor),
            None => Ok(()),
        }
    }

    fn before<P>(&self, read: impl FnOnce() -> Result<Option<Synced<P>>>) -> Result<Before<P>> {
        Ok(Before(if self.linked { read()? } else { None }))
    }

    /// Sends the columns that differ from `before`. A row that had no sync id gets one and
    /// is sent whole.
    fn changed<P: Serialize>(
        &self,
        entity: Entity,
        before: Before<P>,
        after: impl FnOnce() -> Result<Option<Synced<P>>>,
        create: impl FnOnce() -> Result<()>,
    ) -> Result<()> {
        if !self.linked {
            return Ok(());
        }
        let Some(before) = before.0 else {
            return create();
        };
        let Some(after) = after()? else {
            return Ok(());
        };
        let changed = changed_columns(&before.row, &after.row)?;
        if changed.is_empty() {
            return Ok(());
        }
        outbox::put(self.conn, entity, &after.key, self.actor, &changed)
    }

    fn put_whole<P: Serialize>(&self, entity: Entity, row: Option<Synced<P>>) -> Result<()> {
        match row {
            Some(Synced { key, row }) => self.put(entity, &key, &row),
            None => Ok(()),
        }
    }

    fn put(&self, entity: Entity, key: &str, row: &impl Serialize) -> Result<()> {
        outbox::put(self.conn, entity, key, self.actor, &columns(row)?)
    }
}

fn columns(row: &impl Serialize) -> Result<Map<String, Value>> {
    match serde_json::to_value(row)? {
        Value::Object(columns) => Ok(columns),
        _ => Ok(Map::new()),
    }
}

/// The columns of `after` whose values differ from `before`.
fn changed_columns<P: Serialize>(before: &P, after: &P) -> Result<Map<String, Value>> {
    let before = columns(before)?;
    Ok(columns(after)?
        .into_iter()
        .filter(|(column, value)| before.get(column) != Some(value))
        .collect())
}

#[cfg(test)]
mod tests {
    use knoa_core::{FolderColor, Ink, protocol::bookmark_id};
    use serde_json::json;

    use super::*;
    use crate::{
        NewBookmarks, Store,
        test_support::{WORKSPACE, WORKSPACE_KEY, bookmark, import_one, linked, mids, pushed},
    };

    #[test]
    fn local_mode_records_nothing() {
        let mut store = Store::open_in_memory().unwrap();
        let id = import_one(&mut store);
        store.create_workspace("Work", Ink::Moss).unwrap();
        store
            .create_folder(WORKSPACE, "Reading", "", true, FolderColor::default(), None)
            .unwrap();
        store.add_tag(id, "rust", Origin::User).unwrap();
        store.trash(id, 10).unwrap();
        store.empty_trash(WORKSPACE).unwrap();

        assert_eq!(pushed(&store), Vec::<Value>::new());
    }

    #[test]
    fn a_new_bookmark_is_sent_whole_under_its_derived_id() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        import_one(&mut store);

        let [put] = <[Value; 1]>::try_from(pushed(&store)).unwrap();
        let key = bookmark_id(WORKSPACE_KEY, "https://a.dev/").unwrap();
        assert_eq!(put["key"], json!(key));
        assert_eq!(put["data"]["workspace_id"], json!(WORKSPACE_KEY));
        assert_eq!(put["data"]["title"], json!("A"));
        assert!(put["data"].get("status").is_none(), "queued is this Mac's");
        assert!(put["data"].get("image_key").is_none());
    }

    #[test]
    fn an_edit_sends_only_the_columns_it_changed() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let id = import_one(&mut store);
        store.acknowledge(&mids(&store)).unwrap();

        store.set_up_next(id, true).unwrap();
        let mut workspace = store.workspace(WORKSPACE).unwrap().unwrap();
        store.update_workspace(&workspace).unwrap();
        workspace.name = "Home".into();
        workspace.browsers = vec!["helium:Default".into()];
        store.update_workspace(&workspace).unwrap();

        let data: Vec<Value> = pushed(&store)
            .into_iter()
            .map(|m| m["data"].clone())
            .collect();
        assert_eq!(
            data,
            [json!({ "up_next": true }), json!({ "name": "Home" })]
        );
    }

    #[test]
    fn deleting_a_workspace_sends_one_delete_for_all_it_holds() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let work = store.create_workspace("Work", Ink::Moss).unwrap();
        store
            .create_folder(work, "Reading", "", true, FolderColor::default(), None)
            .unwrap();
        store
            .import(
                work,
                "helium:Default",
                &[bookmark("https://a.dev/", "A", 1)],
                NewBookmarks::Organize,
            )
            .unwrap();
        store.acknowledge(&mids(&store)).unwrap();

        store.delete_workspace(work).unwrap();

        let [delete] = <[Value; 1]>::try_from(pushed(&store)).unwrap();
        assert_eq!(
            (&delete["entity"], &delete["op"]),
            (&json!("workspace"), &json!("delete"))
        );
    }
}
