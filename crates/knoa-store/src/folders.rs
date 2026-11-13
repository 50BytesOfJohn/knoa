//! Folders and folder membership.

mod edits;

use knoa_core::{BookmarkId, Folder, FolderColor, FolderIcon, FolderId, Hue, Ink, WorkspaceId};
use rusqlite::params;

use crate::{Result, Store, search, sync::Recorder};

impl Store {
    /// A workspace's folders in sidebar order, with how many bookmarks each holds.
    pub fn folders(&self, workspace: WorkspaceId) -> Result<Vec<Folder>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT f.id, f.name, f.description, f.auto_assign, f.color, f.hue, f.icon,
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
                color: stored_color(row.get(4)?, row.get(5)?),
                icon: row
                    .get::<_, Option<String>>(6)?
                    .as_deref()
                    .and_then(FolderIcon::from_name),
                count: row.get(7)?,
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
        icon: Option<FolderIcon>,
    ) -> Result<FolderId> {
        let (ink, hue) = color_columns(color);
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO folders (workspace_id, name, description, auto_assign, color, hue, icon, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, (SELECT coalesce(max(position), 0) + 1 FROM folders))",
            params![
                workspace.0,
                name,
                description,
                auto_assign,
                ink,
                hue,
                icon.map(FolderIcon::name),
            ],
        )?;
        let id = tx.last_insert_rowid();
        Recorder::user(&tx)?.created_folder(id)?;
        tx.commit()?;
        Ok(FolderId(id))
    }

    pub fn update_folder(&mut self, folder: &Folder) -> Result<()> {
        let (ink, hue) = color_columns(folder.color);
        let tx = self.conn.transaction()?;
        let recorder = Recorder::user(&tx)?;
        let before = recorder.folder_before(folder.id.0)?;
        tx.execute(
            "UPDATE folders SET name = ?2, description = ?3, auto_assign = ?4, color = ?5, hue = ?6,
                icon = ?7 WHERE id = ?1",
            params![
                folder.id.0,
                folder.name,
                folder.description,
                folder.auto_assign,
                ink,
                hue,
                folder.icon.map(FolderIcon::name),
            ],
        )?;
        recorder.folder_changed(folder.id.0, before)?;
        tx.commit()?;
        Ok(())
    }

    /// Deletes the folder. Its bookmarks stay in the library.
    pub fn delete_folder(&mut self, id: FolderId) -> Result<()> {
        let tx = self.conn.transaction()?;
        Recorder::user(&tx)?.deleting_folder(id.0)?;
        tx.execute("DELETE FROM folders WHERE id = ?1", [id.0])?;
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
    /// workspaces. Never removes a link.
    pub(crate) fn add_ai_folders(
        tx: &rusqlite::Transaction<'_>,
        recorder: &Recorder<'_>,
        bookmark: BookmarkId,
        folders: &[(FolderId, f32)],
    ) -> Result<()> {
        for (folder, probability) in folders {
            let added = tx.execute(
                "INSERT INTO bookmark_folders (bookmark_id, folder_id, origin, probability)
                 SELECT ?1, ?2, 'ai', ?3
                 WHERE EXISTS (SELECT 1 FROM folders f JOIN bookmarks b ON b.workspace_id = f.workspace_id
                               WHERE f.id = ?2 AND b.id = ?1)
                   AND NOT EXISTS (SELECT 1 FROM folder_exclusions WHERE bookmark_id = ?1 AND folder_id = ?2)
                 ON CONFLICT DO NOTHING",
                params![bookmark.0, folder.0, probability],
            )?;
            if added > 0 {
                recorder.folder_link(bookmark.0, folder.0)?;
            }
        }
        Ok(())
    }
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
            .create_folder(WORKSPACE, "Reading", "", true, FolderColor::default(), None)
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

    #[test]
    fn a_glyph_this_version_does_not_know_reads_as_the_folder_shape() {
        let mut store = Store::open_in_memory().unwrap();
        let id = store
            .create_folder(
                WORKSPACE,
                "Reading",
                "",
                true,
                FolderColor::default(),
                Some(FolderIcon::Read),
            )
            .unwrap();
        store
            .conn
            .execute("UPDATE folders SET icon = 'rocket' WHERE id = ?1", [id.0])
            .unwrap();

        let folder = store.folders(WORKSPACE).unwrap().remove(0);
        assert_eq!(folder.icon, None);
    }
}
