//! Applying field-level folder edits without replacing unrelated current values.

use knoa_core::{FolderEdits, FolderIcon, FolderId};
use rusqlite::params;

use super::color_columns;
use crate::{Result, Store, sync::Recorder};

impl Store {
    /// Applies only fields changed by the editor, preserving unrelated remote edits.
    pub fn edit_folder(&mut self, id: FolderId, edits: &FolderEdits) -> Result<()> {
        if edits.is_empty() {
            return Ok(());
        }
        let color = edits.color.map(color_columns);
        let tx = self.conn.transaction()?;
        let recorder = Recorder::user(&tx)?;
        let before = recorder.folder_before(id.0)?;
        tx.execute(
            "UPDATE folders SET name = coalesce(?2, name), description = coalesce(?3, description),
                auto_assign = coalesce(?4, auto_assign), color = coalesce(?5, color),
                hue = CASE WHEN ?6 THEN ?7 ELSE hue END,
                icon = CASE WHEN ?8 THEN ?9 ELSE icon END WHERE id = ?1",
            params![
                id.0,
                edits.name,
                edits.description,
                edits.auto_assign,
                color.map(|(ink, _)| ink),
                color.is_some(),
                color.and_then(|(_, hue)| hue),
                edits.icon.is_some(),
                edits.icon.flatten().map(FolderIcon::name)
            ],
        )?;
        recorder.folder_changed(id.0, before)?;
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use knoa_core::{FolderColor, Hue, Ink};

    use super::*;
    use crate::test_support::WORKSPACE;

    #[test]
    fn folder_edits_preserve_remote_fields_and_can_clear_nullable_values() {
        use crate::test_support::{WORKSPACE_KEY, change, linked, mids, pushed};
        use serde_json::json;

        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let id = store
            .create_folder(
                WORKSPACE,
                "Reading",
                "",
                true,
                FolderColor::Hue(Hue::new(120).unwrap()),
                Some(FolderIcon::Read),
            )
            .unwrap();
        let baseline = store.folders(WORKSPACE).unwrap().remove(0);
        let mut edited = baseline.clone();
        edited.name = "Local name".into();
        edited.icon = None;
        edited.color = FolderColor::Ink(Ink::Moss);
        let edits = FolderEdits::between(&baseline, &edited);
        let put = pushed(&store).remove(0);
        let key = put["key"].as_str().unwrap();
        store.acknowledge(&mids(&store)).unwrap();
        let mut row = put["data"].clone();
        row["id"] = json!(key);
        row["workspace_id"] = json!(WORKSPACE_KEY);
        row["description"] = json!("Remote description");
        row["auto_assign"] = json!(false);
        store
            .apply_remote(
                &[change(json!({
                    "entity": "folder", "key": key, "version": 2, "deleted": false, "data": row,
                }))],
                2,
            )
            .unwrap();

        let mut rebased = store.folders(WORKSPACE).unwrap().remove(0);
        edits.apply_to(&mut rebased);
        store.edit_folder(id, &edits).unwrap();

        let saved = store.folders(WORKSPACE).unwrap().remove(0);
        assert_eq!(
            saved, rebased,
            "refreshing and saving must preserve the same intent"
        );
        assert_eq!(saved.name, "Local name");
        assert_eq!(saved.description, "Remote description");
        assert!(!saved.auto_assign);
        assert_eq!(saved.color, edited.color);
        assert_eq!(saved.icon, None);
        assert_eq!(
            pushed(&store)[0]["data"],
            json!({
                "name": "Local name", "color": Ink::Moss.index(), "hue": null, "icon": null,
            })
        );
    }
}
