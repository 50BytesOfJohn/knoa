//! Linking the library to an account, and unlinking it.
//!
//! Linking gives every row its sync id, adopting the account's workspace or folder of the
//! same name, and queues what the server doesn't have yet. It happens in one transaction
//! together with applying everything the server has, so a crash leaves the library either
//! linked and up to date, or local as before.

use std::collections::{HashMap, HashSet};

use knoa_core::{
    Origin,
    protocol::{Change, Changed, RowState, bookmark_id, new_id},
};
use rusqlite::{Connection, OptionalExtension as _, params};

use super::{
    Recorder, apply,
    ids::{self, Table},
};
use crate::{Error, Result, Store};

impl Store {
    /// Links the library to `user_id`'s account. `remote` is everything the server has,
    /// pulled from cursor 0, and `cursor` the pull's last cursor.
    ///
    /// Linking again to the same account changes nothing; the caller pulls as usual.
    /// Another account is refused until [`Store::unlink`].
    pub fn link(&mut self, user_id: &str, remote: &[Change], cursor: i64) -> Result<()> {
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
            "INSERT INTO sync_state (id, user_id) VALUES (1, ?1)",
            [user_id],
        )?;
        let remote_rows = RemoteRows::new(remote);
        let new = New {
            workspaces: adopt_workspaces(&tx, &remote_rows)?,
            folders: adopt_folders(&tx, &remote_rows)?,
            bookmarks: assign_bookmarks(&tx, &remote_rows)?,
        };
        enqueue(&tx, &new)?;
        apply::apply(&tx, remote)?;
        apply::set_cursor(&tx, cursor)?;
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
             UPDATE workspaces SET sync_id = NULL;
             UPDATE folders SET sync_id = NULL;
             UPDATE bookmarks SET sync_id = NULL, image_key = NULL;
             -- Waiting for the cloud organizer, which no longer runs for this library.
             UPDATE bookmarks SET status = 'new'
                 WHERE status = 'queued' AND id NOT IN (SELECT bookmark_id FROM jobs);",
        )?;
        tx.commit()?;
        Ok(())
    }
}

/// What the server already has, as of the end of `remote`.
#[derive(Default)]
struct RemoteRows {
    /// Workspace id by name.
    workspaces: Vec<(String, String)>,
    /// Folder id by workspace id and name.
    folders: Vec<(String, String, String)>,
    bookmarks: HashSet<String>,
}

impl RemoteRows {
    fn new(changes: &[Change]) -> RemoteRows {
        let mut workspaces = HashMap::new();
        let mut folders = HashMap::new();
        let mut bookmarks = HashMap::new();
        for change in changes {
            let key = change.key.clone();
            match &change.row {
                Changed::Workspace(RowState::Live(row)) => {
                    workspaces.insert(key, Some(same_name(&row.name)));
                }
                Changed::Folder(RowState::Live(row)) => {
                    folders.insert(key, Some((row.workspace_id.clone(), same_name(&row.name))));
                }
                Changed::Bookmark(RowState::Live(_)) => {
                    bookmarks.insert(key, true);
                }
                Changed::Workspace(RowState::Deleted) => {
                    workspaces.insert(key, None);
                }
                Changed::Folder(RowState::Deleted) => {
                    folders.insert(key, None);
                }
                Changed::Bookmark(RowState::Deleted) => {
                    bookmarks.insert(key, false);
                }
                _ => {}
            }
        }
        RemoteRows {
            workspaces: live(workspaces).collect(),
            folders: live(folders)
                .map(|(id, (workspace, name))| (id, workspace, name))
                .collect(),
            bookmarks: bookmarks
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
    use knoa_core::{FolderColor, Query};
    use serde_json::{Value, json};

    use super::*;
    use crate::{
        NewBookmarks,
        test_support::{WORKSPACE, WORKSPACE_KEY, bookmark, change, pushed, workspace_row},
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

        store.link("user-1", &[remote], 3).unwrap();

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
        assert_eq!(store.cursor().unwrap(), Some(3));
    }

    #[test]
    fn linking_to_another_account_is_refused_until_unlinked() {
        let mut store = Store::open_in_memory().unwrap();
        store.link("user-1", &[], 1).unwrap();
        store.link("user-1", &[], 1).unwrap();

        assert!(matches!(
            store.link("user-2", &[], 1),
            Err(Error::OtherAccount { .. })
        ));
        store.unlink().unwrap();
        store.link("user-2", &[], 1).unwrap();
        assert_eq!(store.account().unwrap().as_deref(), Some("user-2"));
    }
}
