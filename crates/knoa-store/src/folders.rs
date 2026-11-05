//! Folders and folder membership.

use knoa_core::{BookmarkId, Folder, FolderColor, FolderId, WorkspaceId};
use rusqlite::params;

use crate::{Result, Store, search};

impl Store {
    /// A workspace's folders in sidebar order, with how many bookmarks each holds.
    pub fn folders(&self, workspace: WorkspaceId) -> Result<Vec<Folder>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT f.id, f.name, f.description, f.auto_assign, f.color,
                    (SELECT count(*) FROM bookmark_folders bf JOIN bookmarks b ON b.id = bf.bookmark_id
                     WHERE bf.folder_id = f.id AND b.deleted_at IS NULL)
             FROM folders f WHERE f.workspace_id = ?1 ORDER BY f.position",
        )?;
        let rows = stmt.query_map([workspace.0], |row| {
            Ok(Folder {
                id: FolderId(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                auto_assign: row.get(3)?,
                color: FolderColor(row.get(4)?),
                count: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn create_folder(
        &mut self,
        workspace: WorkspaceId,
        name: &str,
        description: &str,
        auto_assign: bool,
        color: FolderColor,
    ) -> Result<FolderId> {
        self.conn.execute(
            "INSERT INTO folders (workspace_id, name, description, auto_assign, color, position)
             VALUES (?5, ?1, ?2, ?3, ?4, (SELECT coalesce(max(position), 0) + 1 FROM folders))",
            params![name, description, auto_assign, color.0, workspace.0],
        )?;
        Ok(FolderId(self.conn.last_insert_rowid()))
    }

    pub fn update_folder(&mut self, folder: &Folder) -> Result<()> {
        self.conn.execute(
            "UPDATE folders SET name = ?2, description = ?3, auto_assign = ?4, color = ?5 WHERE id = ?1",
            params![folder.id.0, folder.name, folder.description, folder.auto_assign, folder.color.0],
        )?;
        Ok(())
    }

    /// Deletes the folder. Its bookmarks stay in the library.
    pub fn delete_folder(&mut self, id: FolderId) -> Result<()> {
        self.conn
            .execute("DELETE FROM folders WHERE id = ?1", [id.0])?;
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
        search::reindex(&tx, bookmark.0)?;
        tx.commit()?;
        Ok(())
    }

    /// Adds AI folder picks, skipping pairs the user removed and folders of other
    /// workspaces. Never removes a link.
    pub(crate) fn add_ai_folders(
        tx: &rusqlite::Transaction<'_>,
        bookmark: BookmarkId,
        folders: &[(FolderId, f32)],
    ) -> Result<()> {
        for (folder, probability) in folders {
            tx.execute(
                "INSERT INTO bookmark_folders (bookmark_id, folder_id, origin, probability)
                 SELECT ?1, ?2, 'ai', ?3
                 WHERE EXISTS (SELECT 1 FROM folders f JOIN bookmarks b ON b.workspace_id = f.workspace_id
                               WHERE f.id = ?2 AND b.id = ?1)
                   AND NOT EXISTS (SELECT 1 FROM folder_exclusions WHERE bookmark_id = ?1 AND folder_id = ?2)
                 ON CONFLICT DO NOTHING",
                params![bookmark.0, folder.0, probability],
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use knoa_core::{Enrichment, Kind, Query};

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
            .create_folder(WORKSPACE, "Reading", "", true, FolderColor(0))
            .unwrap();
        let job = store.take_jobs(0, 1).unwrap().remove(0);
        let enrichment = Enrichment {
            kind: Kind::Article,
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
}
