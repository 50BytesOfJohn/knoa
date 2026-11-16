//! Folders and folder membership.

mod edits;
mod placement;
mod tree;

use std::collections::HashSet;

use knoa_core::{
    BookmarkId, FolderColor, FolderIcon, FolderId, FolderTree, Hue, Ink, NewFolder, WorkspaceId,
};
use rusqlite::params;

use crate::{Result, Store, search, sync::Recorder};

impl Store {
    /// A workspace's folders as the sidebar shows them, each with how many bookmarks it
    /// and its subfolders hold.
    pub fn folders(&self, workspace: WorkspaceId) -> Result<FolderTree> {
        let tree = tree::load(&self.conn, workspace)?;
        tree::with_counts(&self.conn, tree)
    }

    /// Creates the folder last among its siblings.
    ///
    /// # Errors
    ///
    /// [`crate::Error::FolderPlacement`] when the parent isn't in the workspace or is
    /// already at the deepest level.
    pub fn create_folder(
        &mut self,
        workspace: WorkspaceId,
        folder: &NewFolder,
    ) -> Result<FolderId> {
        let (ink, hue) = color_columns(folder.color);
        let tx = self.conn.transaction()?;
        tree::load(&tx, workspace)?.check_new_child(folder.parent)?;
        tx.execute(
            "INSERT INTO folders (workspace_id, parent_id, name, description, auto_assign, color, hue,
                                  icon, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                workspace.0,
                folder.parent.map(|parent| parent.0),
                folder.name,
                folder.description,
                folder.auto_assign,
                ink,
                hue,
                folder.icon.as_ref().map(FolderIcon::stored),
                placement::next_position(&tx, workspace.0, folder.parent)?,
            ],
        )?;
        let id = tx.last_insert_rowid();
        Recorder::user(&tx)?.created_folder(id)?;
        tx.commit()?;
        Ok(FolderId(id))
    }

    /// Deletes the folder and every folder under it, deepest first. Their bookmarks stay
    /// in the library.
    pub fn delete_folder(&mut self, id: FolderId) -> Result<()> {
        let tx = self.conn.transaction()?;
        let recorder = Recorder::user(&tx)?;
        // A subtree lists each parent before its children, so reversed it ends at `id`.
        for folder in subtree(&tx, id)?.into_iter().rev() {
            recorder.deleting_folder(folder.0)?;
            tx.execute("DELETE FROM folders WHERE id = ?1", [folder.0])?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn add_to_folder(&mut self, bookmark: BookmarkId, folder: FolderId) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM folder_exclusions WHERE bookmark_id = ?1 AND folder_id = ?2",
            params![bookmark.0, folder.0],
        )?;
        tx.execute(
            "INSERT INTO bookmark_folders (bookmark_id, folder_id, origin) VALUES (?1, ?2, 'user')
             ON CONFLICT DO UPDATE SET origin = 'user', probability = NULL",
            params![bookmark.0, folder.0],
        )?;
        Recorder::user(&tx)?.folder_link(bookmark.0, folder.0)?;
        tx.commit()?;
        Ok(())
    }

    /// Takes a bookmark out of a folder and remembers it, so the AI never adds it back.
    /// This is also the one-click undo for a folder the AI picked.
    pub fn remove_from_folder(&mut self, bookmark: BookmarkId, folder: FolderId) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM bookmark_folders WHERE bookmark_id = ?1 AND folder_id = ?2",
            params![bookmark.0, folder.0],
        )?;
        tx.execute(
            "INSERT OR IGNORE INTO folder_exclusions VALUES (?1, ?2)",
            params![bookmark.0, folder.0],
        )?;
        Recorder::user(&tx)?.folder_link(bookmark.0, folder.0)?;
        search::reindex(&tx, bookmark.0)?;
        tx.commit()?;
        Ok(())
    }

    /// Adds AI folder picks, skipping pairs the user removed and folders of other
    /// workspaces. Only the most specific AI link stays in a branch: a pick under an AI
    /// link replaces it, and a pick above any membership is skipped, since the bookmark
    /// already shows there. Links the user made are never touched.
    pub(crate) fn add_ai_folders(
        tx: &rusqlite::Transaction<'_>,
        recorder: &Recorder<'_>,
        bookmark: BookmarkId,
        folders: &[(FolderId, f32)],
    ) -> Result<()> {
        if folders.is_empty() {
            return Ok(());
        }
        let Some(tree) = tree::load_for_bookmark(tx, bookmark)? else {
            return Ok(());
        };
        let mut memberships = folder_memberships(tx, bookmark)?;
        for &(folder, probability) in folders {
            if tree
                .descendants(folder)
                .iter()
                .any(|f| memberships.contains(&f.id))
            {
                continue;
            }
            let added = tx.execute(
                "INSERT INTO bookmark_folders (bookmark_id, folder_id, origin, probability)
                 SELECT ?1, ?2, 'ai', ?3
                 WHERE EXISTS (SELECT 1 FROM folders f JOIN bookmarks b ON b.workspace_id = f.workspace_id
                               WHERE f.id = ?2 AND b.id = ?1)
                   AND NOT EXISTS (SELECT 1 FROM folder_exclusions WHERE bookmark_id = ?1 AND folder_id = ?2)
                 ON CONFLICT DO NOTHING",
                params![bookmark.0, folder.0, probability],
            )?;
            if added == 0 {
                continue;
            }
            memberships.insert(folder);
            recorder.folder_link(bookmark.0, folder.0)?;
            let path = tree.path(folder);
            for ancestor in &path[..path.len().saturating_sub(1)] {
                let removed = tx.execute(
                    "DELETE FROM bookmark_folders
                     WHERE bookmark_id = ?1 AND folder_id = ?2 AND origin = 'ai'",
                    params![bookmark.0, ancestor.id.0],
                )?;
                if removed > 0 {
                    memberships.remove(&ancestor.id);
                    recorder.folder_link(bookmark.0, ancestor.id.0)?;
                }
            }
        }
        Ok(())
    }
}

/// All current memberships, including manual links, read once for a filing run.
fn folder_memberships(
    conn: &rusqlite::Connection,
    bookmark: BookmarkId,
) -> Result<HashSet<FolderId>> {
    let mut stmt =
        conn.prepare_cached("SELECT folder_id FROM bookmark_folders WHERE bookmark_id = ?1")?;
    let rows = stmt.query_map([bookmark.0], |row| row.get(0).map(FolderId))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// The folder and every folder under it; empty when there is no such folder.
pub(crate) fn subtree(conn: &rusqlite::Connection, id: FolderId) -> Result<Vec<FolderId>> {
    let tree = tree::load_around(conn, id)?;
    Ok(tree.map_or_else(Vec::new, |tree| {
        tree.subtree(id).iter().map(|f| f.id).collect()
    }))
}

/// An ink from its stored index. Any number names some ink, so a bad value can't hide a
/// folder or workspace.
pub(crate) fn stored_ink(index: i64) -> Ink {
    Ink::nth(usize::try_from(index).unwrap_or_default())
}

/// A set `hue` wins over the ink; one outside the allowed range falls back to it.
fn stored_color(ink: i64, hue: Option<i64>) -> FolderColor {
    hue.and_then(|degrees| u16::try_from(degrees).ok())
        .and_then(Hue::new)
        .map_or(FolderColor::Ink(stored_ink(ink)), FolderColor::Hue)
}

/// The `color` and `hue` columns. A free hue leaves `color` at the first ink.
fn color_columns(color: FolderColor) -> (usize, Option<u16>) {
    match color {
        FolderColor::Ink(ink) => (ink.index(), None),
        FolderColor::Hue(hue) => (Ink::default().index(), Some(hue.degrees())),
    }
}

#[cfg(test)]
mod tests {
    use knoa_core::{Enrichment, FolderGlyph, FolderIcon, Kind, Query};

    use super::*;
    use crate::{
        NewBookmarks,
        test_support::{WORKSPACE, bookmark},
    };

    #[test]
    fn ai_never_adds_back_a_folder_the_user_removed() {
        let mut store = Store::open_in_memory().unwrap();
        store
            .import(
                WORKSPACE,
                "test",
                &[bookmark("https://a.dev/", "A", 1)],
                NewBookmarks::Organize,
            )
            .unwrap();
        let folder = store
            .create_folder(WORKSPACE, &NewFolder::named("Reading"))
            .unwrap();
        let job = store.take_jobs(0, 1).unwrap().remove(0);
        let enrichment = Enrichment {
            kind: Kind::Article,
            title: None,
            summary: None,
            fields: knoa_core::Fields::new(),
            extras: Vec::new(),
            tags: vec!["rust".into()],
            folders: vec![(folder, 0.9)],
            set_aside: None,
            model: "test".into(),
        };
        store
            .finish_organized(job.bookmark.id, &enrichment, 10)
            .unwrap();
        store.remove_from_folder(job.bookmark.id, folder).unwrap();

        store.queue(job.bookmark.id).unwrap();
        let job = store.take_jobs(0, 1).unwrap().remove(0);
        store
            .finish_organized(job.bookmark.id, &enrichment, 20)
            .unwrap();

        let saved = store.search(&Query::default(), 1).unwrap().remove(0);
        assert_eq!(saved.folders, []);
        assert_eq!(saved.tags, [("rust".to_owned(), knoa_core::Origin::Ai)]);
    }

    #[test]
    fn an_icon_reads_back_as_an_emoji_or_as_the_folder_shape_when_unknown() {
        let mut store = Store::open_in_memory().unwrap();
        let id = store
            .create_folder(
                WORKSPACE,
                &NewFolder {
                    icon: Some(FolderGlyph::Read.into()),
                    ..NewFolder::named("Reading")
                },
            )
            .unwrap();
        store
            .conn
            .execute("UPDATE folders SET icon = 'rocket' WHERE id = ?1", [id.0])
            .unwrap();

        let tree = store.folders(WORKSPACE).unwrap();
        assert_eq!(tree.folders()[0].icon, None);
        store
            .conn
            .execute("UPDATE folders SET icon = '🎨' WHERE id = ?1", [id.0])
            .unwrap();
        let tree = store.folders(WORKSPACE).unwrap();
        assert_eq!(tree.folders()[0].icon, FolderIcon::parse("🎨"));
    }

    /// Files the store's one bookmark as the AI would, into `folders`.
    fn ai_files(store: &mut Store, folders: &[FolderId]) -> Vec<(FolderId, knoa_core::Origin)> {
        let bookmark = store.search(&Query::default(), 1).unwrap().remove(0).id;
        store.queue(bookmark).unwrap();
        let job = store.take_jobs(0, 1).unwrap().remove(0);
        let enrichment = Enrichment {
            kind: Kind::Article,
            title: None,
            summary: None,
            fields: knoa_core::Fields::new(),
            extras: Vec::new(),
            tags: Vec::new(),
            folders: folders.iter().map(|&f| (f, 0.9)).collect(),
            set_aside: None,
            model: "test".into(),
        };
        store
            .finish_organized(job.bookmark.id, &enrichment, 10)
            .unwrap();
        let saved = store.search(&Query::default(), 1).unwrap().remove(0);
        saved.folders.iter().map(|l| (l.folder, l.origin)).collect()
    }

    /// A store with one bookmark and Wallpapers › Anime.
    fn wallpapers() -> (Store, FolderId, FolderId) {
        let mut store = Store::open_in_memory().unwrap();
        store
            .import(
                WORKSPACE,
                "test",
                &[bookmark("https://a.dev/", "A", 1)],
                NewBookmarks::Organize,
            )
            .unwrap();
        let parent = store
            .create_folder(WORKSPACE, &NewFolder::named("Wallpapers"))
            .unwrap();
        let child = store
            .create_folder(
                WORKSPACE,
                &NewFolder {
                    parent: Some(parent),
                    ..NewFolder::named("Anime")
                },
            )
            .unwrap();
        (store, parent, child)
    }

    #[test]
    fn only_the_deepest_ai_pick_in_a_branch_keeps_its_link() {
        use knoa_core::Origin::Ai;
        let (mut store, parent, child) = wallpapers();

        assert_eq!(ai_files(&mut store, &[parent]), [(parent, Ai)]);
        assert_eq!(
            ai_files(&mut store, &[child]),
            [(child, Ai)],
            "a deeper pick replaces it"
        );
        assert_eq!(
            ai_files(&mut store, &[parent]),
            [(child, Ai)],
            "a pick above is skipped"
        );
        assert_eq!(ai_files(&mut store, &[parent, child]), [(child, Ai)]);
    }

    #[test]
    fn ai_skips_ancestors_of_a_manual_membership() {
        use knoa_core::Origin::User;
        let (mut store, parent, child) = wallpapers();
        let bookmark = store.search(&Query::default(), 1).unwrap().remove(0).id;
        store.add_to_folder(bookmark, child).unwrap();

        assert_eq!(ai_files(&mut store, &[parent]), [(child, User)]);
    }

    #[test]
    fn a_deeper_ai_pick_leaves_a_folder_the_user_chose_above_it() {
        use knoa_core::Origin::{Ai, User};
        let (mut store, parent, child) = wallpapers();
        ai_files(&mut store, &[parent]);
        let bookmark = store.search(&Query::default(), 1).unwrap().remove(0).id;
        store.add_to_folder(bookmark, parent).unwrap();

        let mut links = ai_files(&mut store, &[child]);
        links.sort_by_key(|(folder, _)| *folder);

        assert_eq!(links, [(parent, User), (child, Ai)]);
    }
}
