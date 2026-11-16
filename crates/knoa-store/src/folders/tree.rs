//! Reading a workspace's folders into a [`FolderTree`], and counting what each subtree holds.

use std::collections::HashMap;

use knoa_core::{Folder, FolderIcon, FolderId, FolderTree, WorkspaceId};
use rusqlite::{Connection, OptionalExtension as _};
use serde_json::json;

use super::stored_color;
use crate::Result;

/// The workspace's folders, with no counts.
pub(crate) fn load(conn: &Connection, workspace: WorkspaceId) -> Result<FolderTree> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, parent_id, name, description, auto_assign, color, hue, icon
         FROM folders WHERE workspace_id = ?1 ORDER BY position, id",
    )?;
    let rows = stmt.query_map([workspace.0], |row| {
        Ok(Folder {
            id: FolderId(row.get(0)?),
            parent: row.get::<_, Option<i64>>(1)?.map(FolderId),
            name: row.get(2)?,
            description: row.get(3)?,
            auto_assign: row.get(4)?,
            color: stored_color(row.get(5)?, row.get(6)?),
            icon: row
                .get::<_, Option<String>>(7)?
                .as_deref()
                .and_then(FolderIcon::parse),
            count: 0,
        })
    })?;
    Ok(FolderTree::new(rows.collect::<rusqlite::Result<_>>()?))
}

/// The tree of the workspace that holds `folder`; `None` when there is no such folder.
pub(crate) fn load_around(conn: &Connection, folder: FolderId) -> Result<Option<FolderTree>> {
    let workspace: Option<i64> = conn
        .query_row(
            "SELECT workspace_id FROM folders WHERE id = ?1",
            [folder.0],
            |row| row.get(0),
        )
        .optional()?;
    workspace
        .map(|workspace| load(conn, WorkspaceId(workspace)))
        .transpose()
}

/// Sets each folder's count to the bookmarks outside Trash in it or under it, each once.
///
/// One query: every (ancestor, folder) pair of the tree goes in as JSON, so the database
/// counts distinct bookmarks per ancestor without walking parents itself.
pub(crate) fn with_counts(conn: &Connection, tree: FolderTree) -> Result<FolderTree> {
    let pairs: Vec<[i64; 2]> = tree
        .folders()
        .iter()
        .flat_map(|folder| {
            tree.path(folder.id)
                .into_iter()
                .map(move |ancestor| [ancestor.id.0, folder.id.0])
        })
        .collect();
    let mut stmt = conn.prepare_cached(
        "WITH closure (ancestor, folder) AS (
             SELECT value ->> 0, value ->> 1 FROM json_each(?1))
         SELECT c.ancestor, count(DISTINCT bf.bookmark_id)
         FROM closure c
         JOIN bookmark_folders bf ON bf.folder_id = c.folder
         JOIN bookmarks b ON b.id = bf.bookmark_id AND b.deleted_at IS NULL
         GROUP BY c.ancestor",
    )?;
    let counts: HashMap<i64, usize> = stmt
        .query_map([json!(pairs).to_string()], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(tree.with_counts(|id| counts.get(&id.0).copied().unwrap_or_default()))
}

#[cfg(test)]
mod tests {
    use knoa_core::{BookmarkId, NewFolder, Place, Query};

    use crate::{
        NewBookmarks, Store,
        test_support::{WORKSPACE, bookmark},
    };

    /// Wallpapers with Anime and Cities under it; `a` is in both subfolders, `b` in Cities
    /// and `c` in Wallpapers itself.
    fn wallpapers(store: &mut Store) -> (knoa_core::FolderId, [BookmarkId; 3]) {
        let urls = ["https://a.dev/", "https://b.dev/", "https://c.dev/"];
        let new: Vec<_> = (1..)
            .zip(urls)
            .map(|(at, url)| bookmark(url, url, at))
            .collect();
        store
            .import(WORKSPACE, "helium:Default", &new, NewBookmarks::Organize)
            .unwrap();
        let newest_first = store.search(&Query::default(), 3).unwrap();
        let ids = [2, 1, 0].map(|at| newest_first[at].id);
        let parent = store
            .create_folder(WORKSPACE, &NewFolder::named("Wallpapers"))
            .unwrap();
        let child = |store: &mut Store, name| {
            let folder = NewFolder {
                parent: Some(parent),
                ..NewFolder::named(name)
            };
            store.create_folder(WORKSPACE, &folder).unwrap()
        };
        let (anime, cities) = (child(store, "Anime"), child(store, "Cities"));
        for (bookmark, folder) in [(0, anime), (0, cities), (1, cities), (2, parent)] {
            store.add_to_folder(ids[bookmark], folder).unwrap();
        }
        (parent, ids)
    }

    #[test]
    fn a_parent_counts_each_bookmark_of_its_subtree_once() {
        let mut store = Store::open_in_memory().unwrap();
        let (parent, _) = wallpapers(&mut store);

        let tree = store.folders(WORKSPACE).unwrap();
        let counts: Vec<(&str, usize)> = tree
            .folders()
            .iter()
            .map(|f| (f.name.as_str(), f.count))
            .collect();
        assert_eq!(counts, [("Wallpapers", 3), ("Anime", 1), ("Cities", 2)]);
        assert_eq!(tree.path_name(tree.folders()[1].id), "Wallpapers › Anime");
        assert_eq!(tree.depth(parent), 0);
    }

    #[test]
    fn a_parent_folder_shows_the_bookmarks_of_its_subfolders() {
        let mut store = Store::open_in_memory().unwrap();
        let (parent, ids) = wallpapers(&mut store);

        let query = Query::parse("", Some(WORKSPACE), Place::Folder(parent));
        let mut found: Vec<BookmarkId> = store
            .search(&query, 10)
            .unwrap()
            .into_iter()
            .map(|b| b.id)
            .collect();
        found.sort();
        assert_eq!(found, ids);
        assert_eq!(store.count(&query).unwrap(), 3);
    }
}
