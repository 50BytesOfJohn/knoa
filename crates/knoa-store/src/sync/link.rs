//! Linking the library to an account, and unlinking it.
//!
//! Linking gives every row its sync id, adopting the account's workspace or folder of the
//! same name, and queues what the server doesn't have yet, in one transaction. What it
//! matches against is only the account's workspaces, folders and bookmark ids, gathered
//! page by page in [`AccountRows`]; the account's rows themselves arrive afterwards
//! through ordinary pulls from cursor 0. A crash before the link leaves the library local
//! as before, and one after it resumes the pull from the last page applied.

use std::collections::{HashMap, HashSet};

use knoa_core::{
    Origin,
    protocol::{Change, Changed, RowState, bookmark_id, new_id},
};
use rusqlite::{Connection, OptionalExtension as _, params};

use super::{
    Recorder,
    ids::{self, Table},
};
use crate::{Error, Result, Store};

/// What linking needs to know of the account: its workspaces and folders by name, and
/// which bookmarks it has. Built from pulled pages, as of the last one added.
#[derive(Debug, Default)]
pub struct AccountRows {
    /// Workspace name by id; `None` once deleted.
    workspaces: HashMap<String, Option<String>>,
    /// Folder workspace id and name by id; `None` once deleted.
    folders: HashMap<String, Option<(String, String)>>,
    /// Whether each bookmark id is live.
    bookmarks: HashMap<String, bool>,
}

impl AccountRows {
    /// Takes in one pulled page, keeping only what matching needs.
    pub fn add(&mut self, changes: &[Change]) {
        for change in changes {
            let key = change.key.clone();
            match &change.row {
                Changed::Workspace(RowState::Live(row)) => {
                    self.workspaces.insert(key, Some(same_name(&row.name)));
                }
                Changed::Folder(RowState::Live(row)) => {
                    let folder = (row.workspace_id.clone(), same_name(&row.name));
                    self.folders.insert(key, Some(folder));
                }
                Changed::Bookmark(RowState::Live(_)) => {
                    self.bookmarks.insert(key, true);
                }
                Changed::Workspace(RowState::Deleted) => {
                    self.workspaces.insert(key, None);
                }
                Changed::Folder(RowState::Deleted) => {
                    self.folders.insert(key, None);
                }
                Changed::Bookmark(RowState::Deleted) => {
                    self.bookmarks.insert(key, false);
                }
                _ => {}
            }
        }
    }
}

impl Store {
    /// Links the library to `user_id`'s account, matching it against `account`, and sets
    /// the cursor to 0 so the next pulls bring everything the account has.
    ///
    /// Linking again to the same account changes nothing; the caller pulls as usual.
    /// Another account is refused until [`Store::unlink`].
    pub fn link(&mut self, user_id: &str, account: AccountRows) -> Result<()> {
        let tx = self.conn.transaction()?;
        let linked: Option<String> = tx
            .query_row("SELECT user_id FROM sync_state", [], |row| row.get(0))
            .optional()?;
        match linked {
            Some(linked) if linked == user_id => return Ok(()),
            Some(linked) => return Err(Error::OtherAccount { linked }),
            None => {}
        }
        tx.execute(
            "INSERT INTO sync_state (id, user_id, cursor) VALUES (1, ?1, 0)",
            [user_id],
        )?;
        let remote = RemoteRows::new(account);
        let new = New {
            workspaces: adopt_workspaces(&tx, &remote)?,
            folders: adopt_folders(&tx, &remote)?,
            bookmarks: assign_bookmarks(&tx, &remote)?,
        };
        enqueue(&tx, &new)?;
        tx.commit()?;
        Ok(())
    }

    /// Makes the library local again: forgets the account, sync ids, unpushed changes and
    /// server thumbnails. The library itself stays as it is.
    pub fn unlink(&mut self) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute_batch(
            "DELETE FROM sync_state;
             DELETE FROM outbox;
             DELETE FROM sync_inbox;
             DELETE FROM unsent_bookmarks;
             UPDATE workspaces SET sync_id = NULL;
             UPDATE folders SET sync_id = NULL;
             UPDATE bookmarks SET sync_id = NULL, image_key = NULL,
                 pending_image = CASE WHEN image_path IS NOT NULL THEN hex(randomblob(16)) END;
             -- Waiting for the cloud organizer, which no longer runs for this library.
             UPDATE bookmarks SET status = 'new'
                 WHERE status = 'queued' AND id NOT IN (SELECT bookmark_id FROM jobs);",
        )?;
        tx.commit()?;
        Ok(())
    }
}

/// The live rows of [`AccountRows`], in the shape matching uses.
struct RemoteRows {
    /// Workspace id by name.
    workspaces: Vec<(String, String)>,
    /// Folder id by workspace id and name.
    folders: Vec<(String, String, String)>,
    bookmarks: HashSet<String>,
}

impl RemoteRows {
    fn new(account: AccountRows) -> RemoteRows {
        RemoteRows {
            workspaces: live(account.workspaces).collect(),
            folders: live(account.folders)
                .map(|(id, (workspace, name))| (id, workspace, name))
                .collect(),
            bookmarks: account
                .bookmarks
                .into_iter()
                .filter_map(|(id, live)| live.then_some(id))
                .collect(),
        }
    }
}

fn live<T>(rows: HashMap<String, Option<T>>) -> impl Iterator<Item = (String, T)> {
    rows.into_iter()
        .filter_map(|(id, row)| row.map(|row| (id, row)))
}

/// Names that a person would call the same.
fn same_name(name: &str) -> String {
    name.trim().to_lowercase()
}

/// Local rows the server has no counterpart for.
struct New {
    workspaces: Vec<i64>,
    folders: Vec<i64>,
    bookmarks: Vec<i64>,
}

fn adopt_workspaces(conn: &Connection, remote: &RemoteRows) -> Result<Vec<i64>> {
    let local = rows::<(i64, String)>(conn, "SELECT id, name FROM workspaces ORDER BY position")?;
    let mut taken = HashSet::new();
    let mut new = Vec::new();
    for (id, name) in local {
        let name = same_name(&name);
        let adopted = remote
            .workspaces
            .iter()
            .find(|(key, remote)| *remote == name && !taken.contains(key));
        let key = if let Some((key, _)) = adopted {
            key.clone()
        } else {
            new.push(id);
            new_id()
        };
        ids::set_sync_id(conn, Table::Workspaces, id, &key)?;
        taken.insert(key);
    }
    Ok(new)
}

fn adopt_folders(conn: &Connection, remote: &RemoteRows) -> Result<Vec<i64>> {
    let local = rows::<(i64, String, String)>(
        conn,
        "SELECT f.id, w.sync_id, f.name FROM folders f JOIN workspaces w ON w.id = f.workspace_id
         ORDER BY f.position",
    )?;
    let mut taken = HashSet::new();
    let mut new = Vec::new();
    for (id, workspace, name) in local {
        let name = same_name(&name);
        let adopted = remote
            .folders
            .iter()
            .find(|(key, remote_workspace, remote)| {
                *remote_workspace == workspace && *remote == name && !taken.contains(key)
            });
        let key = if let Some((key, ..)) = adopted {
            key.clone()
        } else {
            new.push(id);
            new_id()
        };
        ids::set_sync_id(conn, Table::Folders, id, &key)?;
        taken.insert(key);
    }
    Ok(new)
}

fn assign_bookmarks(conn: &Connection, remote: &RemoteRows) -> Result<Vec<i64>> {
    let local = rows::<(i64, String, String)>(
        conn,
        "SELECT b.id, w.sync_id, b.url FROM bookmarks b JOIN workspaces w ON w.id = b.workspace_id",
    )?;
    let mut new = Vec::new();
    for (id, workspace, url) in local {
        let key = bookmark_id(&workspace, &url).ok_or(Error::InvalidSyncId(workspace))?;
        ids::set_sync_id(conn, Table::Bookmarks, id, &key)?;
        if !remote.bookmarks.contains(&key) {
            new.push(id);
        }
    }
    Ok(new)
}

/// Queues the rows the server lacks, whole, and every tag and folder link, which the server
/// merges with its own: an AI link never replaces a user's.
fn enqueue(conn: &Connection, new: &New) -> Result<()> {
    let user = Recorder::user(conn)?;
    let ai = Recorder::ai(conn)?;
    for id in &new.workspaces {
        user.put_whole_workspace(*id)?;
    }
    for id in &new.folders {
        user.put_whole_folder(*id)?;
    }
    for id in &new.bookmarks {
        user.put_whole_bookmark(*id)?;
        ai.text(*id)?;
    }
    let by = |origin: &str| {
        if origin == Origin::Ai.slug() {
            &ai
        } else {
            &user
        }
    };
    let tags = rows::<(i64, String, String)>(
        conn,
        "SELECT bookmark_id, tag, origin FROM bookmark_tags
         UNION ALL SELECT bookmark_id, tag, 'user' FROM tag_exclusions",
    )?;
    for (bookmark, tag, origin) in tags {
        by(&origin).tag(bookmark, &tag)?;
    }
    let links = rows::<(i64, i64, String)>(
        conn,
        "SELECT bookmark_id, folder_id, origin FROM bookmark_folders
         UNION ALL SELECT bookmark_id, folder_id, 'user' FROM folder_exclusions",
    )?;
    for (bookmark, folder, origin) in links {
        by(&origin).folder_link(bookmark, folder)?;
    }
    Ok(())
}

fn rows<T>(conn: &Connection, sql: &str) -> Result<Vec<T>>
where
    T: for<'r> TryFrom<&'r rusqlite::Row<'r>, Error = rusqlite::Error>,
{
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![], |row| T::try_from(row))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

#[cfg(test)]
mod tests {
    use knoa_core::{FolderColor, Ink, Query};
    use serde_json::{Value, json};

    use super::*;
    use crate::{
        NewBookmarks,
        test_support::{
            WORKSPACE, WORKSPACE_KEY, bookmark, bookmark_row, change, link, mids, pushed,
            workspace_row,
        },
    };

    #[test]
    fn linking_adopts_same_named_rows_and_queues_the_rest() {
        let mut store = Store::open_in_memory().unwrap();
        let reading = store
            .create_folder(WORKSPACE, "Reading", "", true, FolderColor::default(), None)
            .unwrap();
        store
            .import(
                WORKSPACE,
                "helium:Default",
                &[bookmark("https://a.dev/", "A", 1)],
                NewBookmarks::Organize,
            )
            .unwrap();
        let id = store.search(&Query::default(), 1).unwrap()[0].id;
        store.add_tag(id, "rust", Origin::User).unwrap();
        store.add_to_folder(id, reading).unwrap();
        let remote = change(json!({
            "entity": "workspace", "key": WORKSPACE_KEY, "version": 3, "deleted": false,
            "data": workspace_row(WORKSPACE_KEY, " personal "),
        }));

        link(&mut store, "user-1", &[&[remote]]);

        let entities: Vec<Value> = pushed(&store)
            .into_iter()
            .map(|m| m["entity"].clone())
            .collect();
        assert_eq!(
            entities,
            ["folder", "bookmark", "bookmark_tag", "bookmark_folder"]
        );
        let workspace = store.workspace(WORKSPACE).unwrap().unwrap();
        assert_eq!(workspace.name, " personal ", "the account's values win");
        assert_eq!(store.cursor().unwrap(), Some(1));
    }

    #[test]
    fn linking_keeps_the_accounts_exclusion_over_a_local_ai_tag() {
        let mut store = Store::open_in_memory().unwrap();
        let url = "https://a.dev/";
        store
            .import(
                WORKSPACE,
                "helium:Default",
                &[bookmark(url, "A", 1)],
                NewBookmarks::Organize,
            )
            .unwrap();
        let id = store.search(&Query::default(), 1).unwrap()[0].id;
        store.add_tag(id, "rust", Origin::Ai).unwrap();
        let key = bookmark_id(WORKSPACE_KEY, url).unwrap();
        let remote = [
            json!({ "entity": "workspace", "key": WORKSPACE_KEY, "version": 1, "deleted": false,
                    "data": workspace_row(WORKSPACE_KEY, "Personal") }),
            json!({ "entity": "bookmark", "key": key, "version": 2, "deleted": false,
                    "data": bookmark_row(&key, WORKSPACE_KEY, url, "A") }),
            json!({ "entity": "bookmark_tag", "key": format!("{key}/rust"), "version": 3,
                    "deleted": false, "data": { "bookmark_id": key, "tag": "rust",
                    "origin": "user", "excluded": true } }),
        ]
        .map(change);

        link(&mut store, "user-1", &[&remote]);

        assert_eq!(store.search(&Query::default(), 1).unwrap()[0].tags, []);
        let entities: Vec<Value> = pushed(&store)
            .into_iter()
            .map(|m| m["entity"].clone())
            .collect();
        assert!(!entities.contains(&json!("bookmark_tag")), "{entities:?}");
    }

    #[test]
    fn linking_to_another_account_is_refused_until_unlinked() {
        let mut store = Store::open_in_memory().unwrap();
        store.link("user-1", AccountRows::default()).unwrap();
        store.link("user-1", AccountRows::default()).unwrap();

        assert!(matches!(
            store.link("user-2", AccountRows::default()),
            Err(Error::OtherAccount { .. })
        ));
        store.unlink().unwrap();
        store.link("user-2", AccountRows::default()).unwrap();
        assert_eq!(store.account().unwrap().as_deref(), Some("user-2"));
    }

    const WORK_KEY: &str = "1b7d0c2e-4a5f-4c6d-9e8f-0a1b2c3d4e5f";
    const FOLDER_KEY: &str = "0d2b6c1e-3f4a-4b5c-8d6e-7f8091a2b3c4";

    /// The account over two pages: Personal with a Reading folder that the second page
    /// deletes, and a Work workspace.
    fn two_pages() -> [Vec<Change>; 2] {
        let first = [
            json!({ "entity": "workspace", "key": WORKSPACE_KEY, "version": 1, "deleted": false,
                    "data": workspace_row(WORKSPACE_KEY, "Personal") }),
            json!({ "entity": "folder", "key": FOLDER_KEY, "version": 2, "deleted": false,
                    "data": { "id": FOLDER_KEY, "workspace_id": WORKSPACE_KEY,
                    "name": "Reading", "description": "", "auto_assign": true, "color": 2,
                    "hue": null, "icon": null, "position": 1 } }),
        ];
        let second = [
            json!({ "entity": "folder", "key": FOLDER_KEY, "version": 3, "deleted": true }),
            json!({ "entity": "workspace", "key": WORK_KEY, "version": 4, "deleted": false,
                    "data": workspace_row(WORK_KEY, "Work") }),
        ];
        [first.map(change).to_vec(), second.map(change).to_vec()]
    }

    /// Local Personal with a Reading folder, and a Work workspace.
    fn local_library(store: &mut Store) {
        store
            .create_folder(WORKSPACE, "Reading", "", true, FolderColor::default(), None)
            .unwrap();
        store.create_workspace("Work", Ink::Moss).unwrap();
    }

    fn workspace_names(store: &Store) -> Vec<String> {
        store
            .workspaces()
            .unwrap()
            .into_iter()
            .map(|w| w.name)
            .collect()
    }

    #[test]
    fn linking_matches_against_the_account_as_of_its_last_page() {
        let mut store = Store::open_in_memory().unwrap();
        local_library(&mut store);
        let [first, second] = two_pages();

        link(&mut store, "user-1", &[&first, &second]);

        let entities: Vec<Value> = pushed(&store)
            .into_iter()
            .map(|m| m["entity"].clone())
            .collect();
        assert_eq!(entities, ["folder"], "Reading is gone from the account");
        assert_eq!(workspace_names(&store), ["Personal", "Work"]);
        assert_eq!(store.cursor().unwrap(), Some(2));
    }

    #[test]
    fn an_interrupted_link_resumes_from_the_last_page_applied() {
        let mut store = Store::open_in_memory().unwrap();
        local_library(&mut store);
        let [first, second] = two_pages();
        let mut account = AccountRows::default();
        account.add(&first);
        account.add(&second);
        store.link("user-1", account).unwrap();
        store.apply_remote(&first, 1).unwrap();
        let queued = mids(&store);

        // The next launch finds the library linked, and pulls on from its cursor.
        store.link("user-1", AccountRows::default()).unwrap();
        assert_eq!(store.cursor().unwrap(), Some(1));
        store.apply_remote(&second, 2).unwrap();

        assert_eq!(mids(&store), queued);
        assert_eq!(workspace_names(&store), ["Personal", "Work"]);
    }
}
