//! Matching local folders to the account's by workspace and path of names.

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use knoa_core::{MAX_FOLDER_DEPTH, protocol::new_id};
use rusqlite::Connection;

use super::{RemoteRows, live, rows, same_name};
use crate::{
    Result,
    sync::ids::{self, Table},
};

#[derive(Debug)]
pub(super) struct RemoteFolder {
    pub workspace: String,
    pub name: String,
    pub parent: Option<String>,
}

pub(super) fn remote_folder_paths(
    folders: HashMap<String, Option<RemoteFolder>>,
) -> Vec<(String, String, Vec<String>)> {
    let folders: HashMap<String, RemoteFolder> = live(folders).collect();
    let links = folders
        .iter()
        .map(|(key, f)| (key.clone(), (f.name.clone(), f.parent.clone())))
        .collect();
    folders
        .iter()
        .map(|(key, f)| (key.clone(), f.workspace.clone(), path_of(key, &links)))
        .collect()
}

/// The names from the top-level folder down to `key`. A missing parent ends the path. A
/// chain longer than the depth limit, which a cycle also makes, leaves the folder at the
/// top level, as the tree shows it.
fn path_of<K: Eq + Hash + Clone>(
    key: &K,
    folders: &HashMap<K, (String, Option<K>)>,
) -> Vec<String> {
    let mut names = Vec::new();
    let mut at = Some(key);
    while let Some((name, parent)) = at.and_then(|key| folders.get(key)) {
        if names.len() == MAX_FOLDER_DEPTH {
            return names.into_iter().take(1).collect();
        }
        names.push(name.clone());
        at = parent.as_ref();
    }
    names.reverse();
    names
}

/// Adopts the account's folders by workspace and path, parents first, so a child adopted
/// or sent new always names its parent's id.
pub(super) fn adopt_folders(conn: &Connection, remote: &RemoteRows) -> Result<Vec<i64>> {
    let mut taken = HashSet::new();
    let mut new = Vec::new();
    for (id, workspace, path) in local_folder_paths(conn)? {
        let adopted = remote
            .folders
            .iter()
            .find(|(key, remote_workspace, remote)| {
                *remote_workspace == workspace && *remote == path && !taken.contains(key)
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

/// Every local folder with its workspace's sync id and path of names, shallowest first.
fn local_folder_paths(conn: &Connection) -> Result<Vec<(i64, String, Vec<String>)>> {
    let local = rows::<(i64, String, String, Option<i64>)>(
        conn,
        "SELECT f.id, w.sync_id, f.name, f.parent_id FROM folders f
         JOIN workspaces w ON w.id = f.workspace_id ORDER BY f.position, f.id",
    )?;
    let links: HashMap<i64, (String, Option<i64>)> = local
        .iter()
        .map(|(id, _, name, parent)| (*id, (same_name(name), *parent)))
        .collect();
    let mut paths: Vec<_> = local
        .into_iter()
        .map(|(id, workspace, ..)| (id, workspace, path_of(&id, &links)))
        .collect();
    paths.sort_by_key(|(.., path)| path.len());
    Ok(paths)
}
