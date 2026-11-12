//! Turns local changes into outbox mutations, inside the change's own transaction.
//!
//! A store method makes its change as before and tells the recorder what it touched:
//! a row it created or is about to delete, a snapshot of a row before an update, or a
//! tag or folder link whose state may have changed. In Local mode every call is a no-op.

use std::collections::BTreeMap;

use knoa_core::{
    Origin,
    protocol::{
        BookmarkFolder, BookmarkPatch, BookmarkTag, BookmarkText, Entity, FolderPatch,
        WorkspacePatch, bookmark_id, folder_link_key, new_id, tag_key,
    },
};
use rusqlite::{Connection, OptionalExtension as _};
use serde::Serialize;
use serde_json::{Map, Value};

use super::{
    ids::{self, Table},
    linked, outbox,
    rows::{self, Link, Synced},
    unsent,
};
use crate::{Error, Result};

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

    /// The bookmark's tags, to compare with after a change that may touch several.
    pub fn tags_before(&self, bookmark: i64) -> Result<Option<BTreeMap<String, Origin>>> {
        if !self.linked {
            return Ok(None);
        }
        rows::tags(self.conn, bookmark).map(Some)
    }

    /// Records each tag that was added, removed or changed origin since `before`.
    pub fn tags_changed(
        &self,
        bookmark: i64,
        before: Option<BTreeMap<String, Origin>>,
    ) -> Result<()> {
        let Some(before) = before else {
            return Ok(());
        };
        let after = rows::tags(self.conn, bookmark)?;
        let touched = before.keys().chain(after.keys());
        for tag in touched.filter(|tag| before.get(*tag) != after.get(*tag)) {
            self.tag(bookmark, tag)?;
        }
        Ok(())
    }

    /// Records the tag as the bookmark has it now: on it, removed by the user, or gone.
    pub fn tag(&self, bookmark: i64, tag: &str) -> Result<()> {
        if !self.linked {
            return Ok(());
        }
        let Some(bookmark_key) = self.bookmark_key(bookmark)? else {
            return Ok(());
        };
        let key = tag_key(&bookmark_key, tag);
        let (origin, excluded) = match rows::tag(self.conn, bookmark, tag)? {
            Link::On { origin, .. } => (origin, false),
            Link::Excluded => (Origin::User, true),
            Link::Gone => return outbox::delete(self.conn, Entity::BookmarkTag, &key, self.actor),
        };
        let row = BookmarkTag {
            bookmark_id: bookmark_key,
            tag: tag.to_owned(),
            origin,
            excluded,
        };
        self.put(Entity::BookmarkTag, &key, &row)
    }

    /// Records the folder link as the bookmark has it now, like [`Recorder::tag`].
    pub fn folder_link(&self, bookmark: i64, folder: i64) -> Result<()> {
        if !self.linked {
            return Ok(());
        }
        let (Some(bookmark_key), Some(folder_key)) =
            (self.bookmark_key(bookmark)?, self.folder_key(folder)?)
        else {
            return Ok(());
        };
        let key = folder_link_key(&bookmark_key, &folder_key);
        let (origin, probability, excluded) = match rows::folder_link(self.conn, bookmark, folder)?
        {
            Link::On {
                origin,
                probability,
            } => (origin, probability, false),
            Link::Excluded => (Origin::User, None, true),
            Link::Gone => {
                return outbox::delete(self.conn, Entity::BookmarkFolder, &key, self.actor);
            }
        };
        let row = BookmarkFolder {
            bookmark_id: bookmark_key,
            folder_id: folder_key,
            origin,
            probability,
            excluded,
        };
        self.put(Entity::BookmarkFolder, &key, &row)
    }

    /// Records the bookmark's page text, if it has any.
    pub fn text(&self, bookmark: i64) -> Result<()> {
        if !self.linked {
            return Ok(());
        }
        let (Some(key), Some(text)) = (
            self.bookmark_key(bookmark)?,
            rows::page_text(self.conn, bookmark)?,
        ) else {
            return Ok(());
        };
        let row = BookmarkText {
            bookmark_id: key.clone(),
            text,
        };
        self.put(Entity::BookmarkText, &key, &row)
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

    /// The workspace's sync id, minted and sent whole if it had none. `None` when there is
    /// no such workspace.
    fn workspace_key(&self, id: i64) -> Result<Option<String>> {
        if !self.linked {
            return Ok(None);
        }
        if let Some(key) = ids::sync_id(self.conn, Table::Workspaces, id)? {
            return Ok(Some(key));
        }
        let key = new_id();
        if !ids::set_sync_id(self.conn, Table::Workspaces, id, &key)? {
            return Ok(None);
        }
        self.put_whole_workspace(id)?;
        Ok(Some(key))
    }

    fn folder_key(&self, id: i64) -> Result<Option<String>> {
        if !self.linked {
            return Ok(None);
        }
        if let Some(key) = ids::sync_id(self.conn, Table::Folders, id)? {
            return Ok(Some(key));
        }
        let Some(workspace) = self.parent(Table::Folders, id)? else {
            return Ok(None);
        };
        self.workspace_key(workspace)?;
        let key = new_id();
        ids::set_sync_id(self.conn, Table::Folders, id, &key)?;
        self.put_whole_folder(id)?;
        Ok(Some(key))
    }

    /// `None` too while the bookmark waits for room on the account: it goes again whole.
    fn bookmark_key(&self, id: i64) -> Result<Option<String>> {
        if !self.linked || unsent::is_unsent(self.conn, id)? {
            return Ok(None);
        }
        if let Some(key) = ids::sync_id(self.conn, Table::Bookmarks, id)? {
            return Ok(Some(key));
        }
        let Some(workspace) = self.parent(Table::Bookmarks, id)? else {
            return Ok(None);
        };
        let Some(namespace) = self.workspace_key(workspace)? else {
            return Ok(None);
        };
        let url: String =
            self.conn
                .query_row("SELECT url FROM bookmarks WHERE id = ?1", [id], |row| {
                    row.get(0)
                })?;
        let key = bookmark_id(&namespace, &url).ok_or(Error::InvalidSyncId(namespace))?;
        ids::set_sync_id(self.conn, Table::Bookmarks, id, &key)?;
        self.put_whole_bookmark(id)?;
        Ok(Some(key))
    }

    /// The workspace a folder or bookmark belongs to.
    fn parent(&self, table: Table, id: i64) -> Result<Option<i64>> {
        let sql = match table {
            Table::Folders => "SELECT workspace_id FROM folders WHERE id = ?1",
            Table::Bookmarks => "SELECT workspace_id FROM bookmarks WHERE id = ?1",
            Table::Workspaces => return Ok(None),
        };
        Ok(self
            .conn
            .query_row(sql, [id], |row| row.get(0))
            .optional()?)
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
    use knoa_core::{BookmarkId, Enrichment, FolderColor, Ink, Kind, Query};
    use serde_json::json;

    use super::*;
    use crate::{
        NewBookmarks, Store,
        test_support::{WORKSPACE, WORKSPACE_KEY, bookmark, linked, pushed},
    };

    fn organized(tags: &[&str]) -> Enrichment {
        Enrichment {
            kind: Kind::Article,
            summary: Some("About Rust".into()),
            fields: knoa_core::Fields::new(),
            extras: Vec::new(),
            tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
            folders: Vec::new(),
            set_aside: None,
            model: "test".into(),
        }
    }

    /// Imports one bookmark for organizing and returns its id.
    fn import_one(store: &mut Store) -> BookmarkId {
        store
            .import(
                WORKSPACE,
                "helium:Default",
                &[bookmark("https://a.dev/", "A", 1)],
                NewBookmarks::Organize,
            )
            .unwrap();
        store.search(&Query::default(), 1).unwrap()[0].id
    }

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
    fn a_rerun_sends_only_the_tags_that_changed() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let id = import_one(&mut store);
        store.take_jobs(0, 1).unwrap();
        store
            .finish_organized(id, &organized(&["rust", "web"]), 10)
            .unwrap();
        store.acknowledge(&mids(&store)).unwrap();

        store.queue(id).unwrap();
        store.take_jobs(0, 1).unwrap();
        store
            .finish_organized(id, &organized(&["web", "cli"]), 20)
            .unwrap();

        let tags: Vec<(String, String, Value)> = pushed(&store)
            .into_iter()
            .filter(|m| m["entity"] == "bookmark_tag")
            .map(|m| {
                (
                    m["key"]
                        .as_str()
                        .unwrap()
                        .rsplit('/')
                        .next()
                        .unwrap()
                        .to_owned(),
                    m["op"].as_str().unwrap().to_owned(),
                    m["actor"].clone(),
                )
            })
            .collect();
        assert_eq!(
            tags,
            [
                ("rust".to_owned(), "delete".to_owned(), json!("ai")),
                ("cli".to_owned(), "put".to_owned(), json!("ai")),
            ]
        );
    }

    #[test]
    fn removing_a_tag_sends_it_excluded() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let id = import_one(&mut store);
        store.add_tag(id, "rust", Origin::Ai).unwrap();
        store.acknowledge(&mids(&store)).unwrap();

        store.remove_tag(id, "rust").unwrap();

        let [put] = <[Value; 1]>::try_from(pushed(&store)).unwrap();
        assert_eq!(put["actor"], json!("user"));
        assert_eq!(put["data"]["excluded"], json!(true));
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

    fn mids(store: &Store) -> Vec<String> {
        store
            .next_push()
            .unwrap()
            .into_iter()
            .map(|m| m.mid)
            .collect()
    }
}
