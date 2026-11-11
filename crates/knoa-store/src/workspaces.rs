//! Workspaces, the browser profiles each one syncs, and their tag lists.

use knoa_core::{Ink, Screening, TagList, Workspace, WorkspaceId};
use rusqlite::{Transaction, params};

use crate::{Result, Store, folders::stored_ink, tag_list};

impl Store {
    /// All workspaces in switcher order. There is always at least one.
    pub fn workspaces(&self) -> Result<Vec<Workspace>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT w.id, w.name, w.color, w.instructions, w.screening, w.guideline, w.opens_in,
                    w.only_listed_tags,
                    (SELECT json_group_array(browser) FROM workspace_browsers WHERE workspace_id = w.id)
             FROM workspaces w ORDER BY w.position",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                Workspace {
                    id: WorkspaceId(row.get(0)?),
                    name: row.get(1)?,
                    color: stored_ink(row.get(2)?),
                    browsers: Vec::new(),
                    opens_in: row.get(6)?,
                    instructions: row.get(3)?,
                    screening: Screening {
                        enabled: row.get(4)?,
                        guideline: row.get(5)?,
                    },
                    tag_list: TagList {
                        tags: Vec::new(),
                        only_these: row.get(7)?,
                    },
                },
                row.get::<_, String>(8)?,
            ))
        })?;
        let mut workspaces = rows
            .map(|row| {
                let (mut workspace, browsers) = row?;
                workspace.browsers = serde_json::from_str(&browsers)?;
                workspace.browsers.sort();
                Ok(workspace)
            })
            .collect::<Result<Vec<Workspace>>>()?;
        for workspace in &mut workspaces {
            workspace.tag_list.tags = tag_list::load(&self.conn, workspace.id)?;
        }
        Ok(workspaces)
    }

    pub fn workspace(&self, id: WorkspaceId) -> Result<Option<Workspace>> {
        Ok(self.workspaces()?.into_iter().find(|w| w.id == id))
    }

    /// Creates a workspace that syncs nothing until the user picks browsers for it.
    pub fn create_workspace(&mut self, name: &str, color: Ink) -> Result<WorkspaceId> {
        self.conn.execute(
            "INSERT INTO workspaces (name, color, position)
             VALUES (?1, ?2, (SELECT coalesce(max(position), 0) + 1 FROM workspaces))",
            params![name, color.index()],
        )?;
        Ok(WorkspaceId(self.conn.last_insert_rowid()))
    }

    pub fn update_workspace(&mut self, workspace: &Workspace) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "UPDATE workspaces SET name = ?2, color = ?3, instructions = ?4, screening = ?5,
                guideline = ?6, opens_in = ?7, only_listed_tags = ?8 WHERE id = ?1",
            params![
                workspace.id.0,
                workspace.name,
                workspace.color.index(),
                workspace.instructions,
                workspace.screening.enabled,
                workspace.screening.guideline,
                workspace.opens_in,
                workspace.tag_list.only_these,
            ],
        )?;
        set_browsers(&tx, workspace.id, &workspace.browsers)?;
        tag_list::replace(&tx, workspace.id, &workspace.tag_list.tags)?;
        tx.commit()?;
        Ok(())
    }

    /// Deletes a workspace with its bookmarks and folders. The last one can't be deleted.
    pub fn delete_workspace(&mut self, id: WorkspaceId) -> Result<bool> {
        let tx = self.conn.transaction()?;
        let count: usize = tx.query_row("SELECT count(*) FROM workspaces", [], |row| row.get(0))?;
        if count <= 1 {
            return Ok(false);
        }
        // The full-text index is a virtual table, outside the foreign-key cascade.
        tx.execute(
            "DELETE FROM bookmarks_fts WHERE rowid IN (SELECT id FROM bookmarks WHERE workspace_id = ?1)",
            [id.0],
        )?;
        tx.execute("DELETE FROM workspaces WHERE id = ?1", [id.0])?;
        tx.commit()?;
        Ok(true)
    }
}

fn set_browsers(tx: &Transaction<'_>, id: WorkspaceId, browsers: &[String]) -> Result<()> {
    tx.execute(
        "DELETE FROM workspace_browsers WHERE workspace_id = ?1",
        [id.0],
    )?;
    for browser in browsers {
        tx.execute(
            "INSERT OR IGNORE INTO workspace_browsers VALUES (?1, ?2)",
            params![id.0, browser],
        )?;
    }
    Ok(())
}
