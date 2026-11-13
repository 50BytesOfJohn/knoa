//! Workspaces, the browser profiles each one syncs, and their tag lists.

use knoa_core::{Ink, Screening, TagList, Workspace, WorkspaceEdits, WorkspaceId};
use rusqlite::{Connection, Transaction, params};

use crate::{
    Result, Store,
    folders::stored_ink,
    sync::{self, Recorder},
    tag_list,
};

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
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO workspaces (name, color, position)
             VALUES (?1, ?2, (SELECT coalesce(max(position), 0) + 1 FROM workspaces))",
            params![name, color.index()],
        )?;
        let id = tx.last_insert_rowid();
        Recorder::user(&tx)?.created_workspace(id)?;
        sync::apply_held(&tx)?;
        tx.commit()?;
        Ok(WorkspaceId(id))
    }

    pub fn update_workspace(&mut self, workspace: &Workspace) -> Result<()> {
        let tx = self.conn.transaction()?;
        let recorder = Recorder::user(&tx)?;
        let before = recorder.workspace_before(workspace.id.0)?;
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
        recorder.workspace_changed(workspace.id.0, before)?;
        tx.commit()?;
        Ok(())
    }

    /// Applies only fields changed by the editor, against the current stored workspace.
    pub fn edit_workspace(&mut self, id: WorkspaceId, edits: &WorkspaceEdits) -> Result<()> {
        if edits.is_empty() {
            return Ok(());
        }
        let tx = self.conn.transaction()?;
        let recorder = Recorder::user(&tx)?;
        let before = recorder.workspace_before(id.0)?;
        tx.execute(
            "UPDATE workspaces SET name = coalesce(?2, name), color = coalesce(?3, color),
                instructions = coalesce(?4, instructions), screening = coalesce(?5, screening),
                guideline = coalesce(?6, guideline),
                opens_in = CASE WHEN ?7 THEN ?8 ELSE opens_in END,
                only_listed_tags = coalesce(?9, only_listed_tags) WHERE id = ?1",
            params![
                id.0,
                edits.name,
                edits.color.map(Ink::index),
                edits.instructions,
                edits.screening,
                edits.guideline,
                edits.opens_in.is_some(),
                edits.opens_in.as_ref().and_then(|value| value.as_deref()),
                edits.only_listed_tags
            ],
        )?;
        if let Some(browsers) = &edits.browsers {
            set_browsers(&tx, id, browsers)?;
        }
        if let Some(tags) = &edits.tags {
            tag_list::replace(&tx, id, tags)?;
        }
        recorder.workspace_changed(id.0, before)?;
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
        Recorder::user(&tx)?.deleting_workspace(id.0)?;
        delete(&tx, id.0)?;
        tx.commit()?;
        Ok(true)
    }
}

/// Deletes a workspace with everything in it.
pub(crate) fn delete(conn: &Connection, id: i64) -> Result<()> {
    // The full-text index is a virtual table, outside the foreign-key cascade.
    conn.execute(
        "DELETE FROM bookmarks_fts WHERE rowid IN (SELECT id FROM bookmarks WHERE workspace_id = ?1)",
        [id],
    )?;
    conn.execute("DELETE FROM workspaces WHERE id = ?1", [id])?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use knoa_core::ListedTag;
    use serde_json::json;

    use super::*;
    use crate::test_support::{WORKSPACE, WORKSPACE_KEY, change, linked, pushed, workspace_row};

    #[test]
    fn workspace_edits_preserve_remote_fields_and_untouched_edits_record_nothing() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let baseline = store.workspace(WORKSPACE).unwrap().unwrap();
        let mut edited = baseline.clone();
        edited.name = "Local name".into();
        edited.screening.enabled = true;
        let edits = WorkspaceEdits::between(&baseline, &edited);
        let mut remote = workspace_row(WORKSPACE_KEY, "Personal");
        remote["instructions"] = json!("Remote instructions");
        remote["guideline"] = json!("Remote guideline");
        remote["only_listed_tags"] = json!(true);
        remote["tag_list"] = json!([{ "tag": "remote", "description": "Remote hint" }]);
        store
            .apply_remote(
                &[change(json!({
                    "entity": "workspace", "key": WORKSPACE_KEY, "version": 2, "deleted": false,
                    "data": remote,
                }))],
                2,
            )
            .unwrap();

        let mut rebased = store.workspace(WORKSPACE).unwrap().unwrap();
        edits.apply_to(&mut rebased);
        store.edit_workspace(WORKSPACE, &edits).unwrap();

        let saved = store.workspace(WORKSPACE).unwrap().unwrap();
        assert_eq!(
            saved, rebased,
            "refreshing the editor and saving must preserve the same intent"
        );
        assert_eq!(saved.name, edited.name);
        assert!(saved.screening.enabled);
        assert_eq!(saved.instructions, "Remote instructions");
        assert_eq!(saved.screening.guideline, "Remote guideline");
        assert!(saved.tag_list.only_these);
        assert_eq!(
            saved.tag_list.tags,
            [ListedTag {
                name: "remote".into(),
                description: "Remote hint".into()
            }]
        );
        assert_eq!(
            pushed(&store)[0]["data"],
            json!({"name": "Local name", "screening": true})
        );
    }
}
