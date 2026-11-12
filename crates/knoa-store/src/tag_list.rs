//! Each workspace's tag list: the tags the AI should reach for, with optional hints.

use knoa_core::{
    ListedTag, MAX_LISTED_TAGS, MAX_TAG_DESCRIPTION_CHARS, WorkspaceId, normalize_tag, tag_fits,
};
use rusqlite::{Connection, params};

use crate::{Result, Store, sync::Recorder};

impl Store {
    /// Adds a tag to the workspace's list. A full list, or a tag already listed, is left
    /// as it is.
    pub fn list_tag(&mut self, workspace: WorkspaceId, tag: &str) -> Result<()> {
        let tag = normalize_tag(tag);
        if !tag_fits(&tag) {
            return Ok(());
        }
        let tx = self.conn.transaction()?;
        let listed = load(&tx, workspace)?;
        if listed.len() < MAX_LISTED_TAGS && !listed.iter().any(|t| t.name == tag) {
            let recorder = Recorder::user(&tx)?;
            let before = recorder.workspace_before(workspace.0)?;
            insert(&tx, workspace, &tag, "")?;
            recorder.workspace_changed(workspace.0, before)?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Takes a tag off the list. Bookmarks keep it. Emptying the list also lets the AI
    /// add its own tags again, so a disabled switch never shows on.
    pub fn unlist_tag(&mut self, workspace: WorkspaceId, tag: &str) -> Result<()> {
        let tx = self.conn.transaction()?;
        let recorder = Recorder::user(&tx)?;
        let before = recorder.workspace_before(workspace.0)?;
        tx.execute(
            "DELETE FROM workspace_tags WHERE workspace_id = ?1 AND tag = ?2",
            params![workspace.0, normalize_tag(tag)],
        )?;
        tx.execute(
            "UPDATE workspaces SET only_listed_tags = 0 WHERE id = ?1
             AND NOT EXISTS (SELECT 1 FROM workspace_tags WHERE workspace_id = ?1)",
            [workspace.0],
        )?;
        recorder.workspace_changed(workspace.0, before)?;
        tx.commit()?;
        Ok(())
    }
}

/// The workspace's list, sorted by name.
pub(crate) fn load(conn: &Connection, workspace: WorkspaceId) -> Result<Vec<ListedTag>> {
    let mut stmt = conn.prepare_cached(
        "SELECT tag, description FROM workspace_tags WHERE workspace_id = ?1 ORDER BY tag",
    )?;
    let rows = stmt.query_map([workspace.0], |row| {
        Ok(ListedTag {
            name: row.get(0)?,
            description: row.get(1)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// Replaces the list, spelling each tag as tags are spelled everywhere and keeping it
/// within its limits.
pub(crate) fn replace(tx: &Connection, workspace: WorkspaceId, tags: &[ListedTag]) -> Result<()> {
    tx.execute(
        "DELETE FROM workspace_tags WHERE workspace_id = ?1",
        [workspace.0],
    )?;
    let mut names: Vec<String> = Vec::new();
    for tag in tags {
        let name = normalize_tag(&tag.name);
        if !tag_fits(&name) || names.contains(&name) || names.len() >= MAX_LISTED_TAGS {
            continue;
        }
        let description: String = tag
            .description
            .trim()
            .chars()
            .take(MAX_TAG_DESCRIPTION_CHARS)
            .collect();
        insert(tx, workspace, &name, &description)?;
        names.push(name);
    }
    Ok(())
}

fn insert(conn: &Connection, workspace: WorkspaceId, tag: &str, description: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO workspace_tags (workspace_id, tag, description) VALUES (?1, ?2, ?3)",
        params![workspace.0, tag, description],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use knoa_core::MAX_TAG_CHARS;
    use serde_json::json;

    use super::*;
    use crate::test_support::{WORKSPACE, WORKSPACE_KEY, change, linked, workspace_row};

    #[test]
    fn a_full_tag_list_takes_no_more_tags() {
        let mut store = Store::open_in_memory().unwrap();
        for index in 0..MAX_LISTED_TAGS {
            store.list_tag(WORKSPACE, &format!("tag {index}")).unwrap();
        }
        store.list_tag(WORKSPACE, "one-more").unwrap();

        let list = store.workspace(WORKSPACE).unwrap().unwrap().tag_list;
        assert_eq!(list.tags.len(), MAX_LISTED_TAGS);
        assert!(list.contains("tag-0") && !list.contains("one-more"));
    }

    #[test]
    fn a_pulled_tag_list_keeps_a_long_non_ascii_tag() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let tag = "zażółć-gęślą-jaźń-źdźbło-łąka-ćma";
        assert!(tag.chars().count() <= MAX_TAG_CHARS && tag.len() > MAX_TAG_CHARS);
        let mut row = workspace_row(WORKSPACE_KEY, "Personal");
        row["tag_list"] = json!([{ "tag": tag, "description": "" }]);
        let pulled = change(json!({ "entity": "workspace", "key": WORKSPACE_KEY,
                                    "version": 2, "deleted": false, "data": row }));

        store.apply_remote(&[pulled], 2).unwrap();

        let list = store.workspace(WORKSPACE).unwrap().unwrap().tag_list;
        assert!(list.contains(tag));
    }
}
