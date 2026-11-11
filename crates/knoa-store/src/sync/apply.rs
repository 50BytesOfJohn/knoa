//! Applying pulled changes to the local library.
//!
//! Pulled changes never go to the outbox. A local change still waiting to be pushed wins
//! over the pulled value of the same column: the server hasn't seen it yet, and will take
//! it when it arrives. Pulls don't promise parents before children, so every change waits
//! in `sync_inbox` until its parent is here, across pulls if need be.

mod bookmark;
mod columns;
mod folder;
mod links;
mod workspace;

use knoa_core::protocol::{Change, Changed, Entity, RowState};
use rusqlite::{Connection, params};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use super::{linked, outbox::Pending};
use crate::{Error, Result, Store};

impl Store {
    /// Applies one page of pulled changes and moves the cursor to `cursor`, together.
    pub fn apply_remote(&mut self, changes: &[Change], cursor: i64) -> Result<()> {
        let tx = self.conn.transaction()?;
        if !linked(&tx)? {
            return Err(Error::NotLinked);
        }
        apply(&tx, changes)?;
        set_cursor(&tx, cursor)?;
        tx.commit()?;
        Ok(())
    }
}

/// Whether a change could be applied, or waits for its parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Outcome {
    Done,
    WaitsForParent,
}

pub(super) fn apply(conn: &Connection, changes: &[Change]) -> Result<()> {
    for change in changes {
        hold(conn, change)?;
    }
    // Parents before children, so a whole page applies in one pass.
    for entity in Entity::ALL {
        for change in held(conn, entity)? {
            if apply_one(conn, change.clone())? == Outcome::Done {
                release(conn, entity, &change.key)?;
            }
        }
    }
    Ok(())
}

pub(super) fn set_cursor(conn: &Connection, cursor: i64) -> Result<()> {
    conn.execute("UPDATE sync_state SET cursor = ?1", [cursor])?;
    Ok(())
}

fn apply_one(conn: &Connection, change: Change) -> Result<Outcome> {
    let key = change.key.as_str();
    match change.row {
        Changed::Workspace(RowState::Live(row)) => workspace::put(conn, key, row),
        Changed::Workspace(RowState::Deleted) => workspace::delete(conn, key),
        Changed::Folder(RowState::Live(row)) => folder::put(conn, key, row),
        Changed::Folder(RowState::Deleted) => folder::delete(conn, key),
        Changed::Bookmark(RowState::Live(row)) => bookmark::put(conn, key, row),
        Changed::Bookmark(RowState::Deleted) => bookmark::delete(conn, key),
        Changed::BookmarkTag(RowState::Live(row)) => links::put_tag(conn, key, &row),
        Changed::BookmarkTag(RowState::Deleted) => links::delete_tag(conn, key),
        Changed::BookmarkFolder(RowState::Live(row)) => links::put_folder_link(conn, key, &row),
        Changed::BookmarkFolder(RowState::Deleted) => links::delete_folder_link(conn, key),
        Changed::BookmarkText(RowState::Live(row)) => links::put_text(conn, key, &row.text),
        Changed::BookmarkText(RowState::Deleted) => links::delete_text(conn, key),
    }
}

/// Keeps the change until it is applied, replacing an older one for the same row.
fn hold(conn: &Connection, change: &Change) -> Result<()> {
    conn.execute(
        "INSERT INTO sync_inbox (entity, key, version, change) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT (entity, key) DO UPDATE SET version = excluded.version, change = excluded.change
         WHERE excluded.version >= sync_inbox.version",
        params![
            change.entity().slug(),
            change.key,
            change.version,
            serde_json::to_string(change)?
        ],
    )?;
    Ok(())
}

fn held(conn: &Connection, entity: Entity) -> Result<Vec<Change>> {
    let mut stmt =
        conn.prepare_cached("SELECT change FROM sync_inbox WHERE entity = ?1 ORDER BY version")?;
    let rows = stmt.query_map([entity.slug()], |row| row.get::<_, String>(0))?;
    rows.map(|change| Ok(serde_json::from_str(&change?)?))
        .collect()
}

fn release(conn: &Connection, entity: Entity, key: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM sync_inbox WHERE entity = ?1 AND key = ?2",
        params![entity.slug(), key],
    )?;
    Ok(())
}

/// Drops held changes that can never apply because their parent was deleted. `condition`
/// is SQL over `sync_inbox` with `?1` as the parent's key.
fn release_orphans(conn: &Connection, condition: &str, parent: &str) -> Result<()> {
    conn.execute(
        &format!("DELETE FROM sync_inbox WHERE {condition}"),
        [parent],
    )?;
    Ok(())
}

/// The row as a patch without the columns a pending local change sets.
fn rebased<P: Serialize + DeserializeOwned>(patch: P, pending: &Pending) -> Result<P> {
    let Value::Object(mut columns) = serde_json::to_value(patch)? else {
        unreachable!("patches serialize as objects");
    };
    columns.retain(|column, _| !pending.columns.contains(column));
    Ok(serde_json::from_value(Value::Object(columns))?)
}

#[cfg(test)]
mod tests {
    use knoa_core::{
        Enrichment, Kind, NewBookmark, Origin, Place, Query, Status, protocol::bookmark_id,
    };
    use serde_json::Value;
    use serde_json::json;

    use super::*;
    use crate::{
        NewBookmarks,
        test_support::{WORKSPACE, WORKSPACE_KEY, bookmark, bookmark_row, change, linked, pushed},
    };

    const URL: &str = "https://a.dev/";

    fn key() -> String {
        bookmark_id(WORKSPACE_KEY, URL).unwrap()
    }

    fn pulled_bookmark(version: i64, title: &str) -> Change {
        change(
            json!({ "entity": "bookmark", "key": key(), "version": version, "deleted": false,
                       "data": bookmark_row(&key(), WORKSPACE_KEY, URL, title) }),
        )
    }

    fn pulled_tag(version: i64, tag: &str, excluded: bool) -> Change {
        change(
            json!({ "entity": "bookmark_tag", "key": format!("{}/{tag}", key()),
                       "version": version, "deleted": false,
                       "data": { "bookmark_id": key(), "tag": tag, "origin": "user", "excluded": excluded } }),
        )
    }

    fn tombstone(entity: &str, key: &str, version: i64) -> Change {
        change(json!({ "entity": entity, "key": key, "version": version, "deleted": true }))
    }

    fn only_bookmark(store: &Store) -> Option<knoa_core::Bookmark> {
        store
            .search(&Query::default(), 2)
            .unwrap()
            .into_iter()
            .next()
    }

    /// As if the server took everything waiting.
    fn push_all(store: &mut Store) {
        let mids: Vec<String> = store
            .pending(100)
            .unwrap()
            .into_iter()
            .map(|m| m.mid)
            .collect();
        store.acknowledge(&mids).unwrap();
    }

    fn import(store: &mut Store, title: &str) {
        let links = [bookmark(URL, title, 1)];
        store
            .import(WORKSPACE, "helium:Default", &links, NewBookmarks::Organize)
            .unwrap();
    }

    #[test]
    fn a_pulled_bookmark_appears_and_is_not_pushed_back() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let folder = "0d2b6c1e-3f4a-4b5c-8d6e-7f8091a2b3c4";
        let changes = [
            change(
                json!({ "entity": "bookmark_folder", "key": format!("{}/{folder}", key()),
                "version": 2, "deleted": false, "data": { "bookmark_id": key(), "folder_id": folder,
                "origin": "ai", "probability": 0.9, "excluded": false } }),
            ),
            pulled_tag(3, "rust", false),
            pulled_bookmark(4, "Rust things"),
            change(
                json!({ "entity": "folder", "key": folder, "version": 5, "deleted": false,
                "data": { "id": folder, "workspace_id": WORKSPACE_KEY, "name": "Reading",
                "description": "", "auto_assign": true, "color": 2, "hue": null, "icon": null,
                "position": 1 } }),
            ),
        ];

        store.apply_remote(&changes, 5).unwrap();

        let found = store
            .search(&Query::parse("rust things", None, Place::Everything), 2)
            .unwrap();
        let [saved] = <[knoa_core::Bookmark; 1]>::try_from(found).unwrap();
        assert_eq!(saved.tags, [("rust".to_owned(), Origin::User)]);
        assert_eq!(saved.folders.len(), 1);
        assert_eq!(pushed(&store), Vec::<Value>::new());
        assert_eq!(store.cursor().unwrap(), Some(5));
    }

    #[test]
    fn a_change_pulled_before_its_parent_waits_for_it() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let work = "a1b2c3d4-0000-4000-8000-000000000001";
        let id = bookmark_id(work, URL).unwrap();
        let orphan = change(
            json!({ "entity": "bookmark", "key": id, "version": 2, "deleted": false,
                                    "data": bookmark_row(&id, work, URL, "A") }),
        );
        store.apply_remote(&[orphan], 2).unwrap();
        assert_eq!(store.count(&Query::default()).unwrap(), 0);

        let parent = change(
            json!({ "entity": "workspace", "key": work, "version": 3, "deleted": false,
            "data": crate::test_support::workspace_row(work, "Work") }),
        );
        store.apply_remote(&[parent], 3).unwrap();

        assert_eq!(store.count(&Query::default()).unwrap(), 1);
    }

    #[test]
    fn a_pending_local_edit_wins_over_the_pull() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        import(&mut store, "A");
        push_all(&mut store);
        let id = only_bookmark(&store).unwrap().id;
        store.set_up_next(id, true).unwrap();

        store
            .apply_remote(&[pulled_bookmark(2, "Renamed")], 2)
            .unwrap();

        let saved = only_bookmark(&store).unwrap();
        assert_eq!(saved.title, "Renamed");
        assert!(saved.up_next, "not pushed yet, so still the user's");
        assert_eq!(
            saved.status,
            Status::Queued,
            "this Mac's queue keeps its own"
        );
    }

    #[test]
    fn a_pulled_tombstone_deletes_the_bookmark_and_keeps_the_browser_from_restoring_it() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        import(&mut store, "A");
        let id = only_bookmark(&store).unwrap().id;
        store.add_tag(id, "rust", Origin::User).unwrap();
        push_all(&mut store);

        store
            .apply_remote(&[tombstone("bookmark", &key(), 2)], 2)
            .unwrap();
        import(&mut store, "A");

        assert!(only_bookmark(&store).is_none());
        assert_eq!(store.tags(WORKSPACE).unwrap(), []);
    }

    #[test]
    fn a_pulled_exclusion_keeps_the_ai_from_adding_the_tag() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        import(&mut store, "A");
        let id = only_bookmark(&store).unwrap().id;

        store
            .apply_remote(&[pulled_tag(2, "rust", true)], 2)
            .unwrap();
        store.take_jobs(0, 1).unwrap();
        let enrichment = Enrichment {
            kind: Kind::Article,
            summary: None,
            fields: knoa_core::Fields::new(),
            extras: Vec::new(),
            tags: vec!["rust".into()],
            folders: Vec::new(),
            set_aside: None,
            model: "test".into(),
        };
        store.finish_organized(id, &enrichment, 10).unwrap();

        assert_eq!(only_bookmark(&store).unwrap().tags, []);
    }

    #[test]
    fn a_title_from_another_mac_survives_this_macs_next_import() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        import(&mut store, "From this Mac");
        push_all(&mut store);

        store
            .apply_remote(&[pulled_bookmark(2, "From the other Mac")], 2)
            .unwrap();
        import(&mut store, "From this Mac");
        assert_eq!(only_bookmark(&store).unwrap().title, "From the other Mac");
        assert_eq!(pushed(&store), Vec::<Value>::new());

        let renamed: NewBookmark = bookmark(URL, "Renamed here", 1);
        store
            .import(
                WORKSPACE,
                "helium:Default",
                &[renamed],
                NewBookmarks::Organize,
            )
            .unwrap();
        assert_eq!(only_bookmark(&store).unwrap().title, "Renamed here");
    }
}
