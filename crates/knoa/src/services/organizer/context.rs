//! What the bookmark's workspace tells the AI.

use knoa_ai::FolderInput;
use knoa_core::{TagList, WorkspaceId};

use crate::state::library::{SharedStore, lock};

/// The workspace's instructions, guideline, tag list and the folders it may file into.
pub(super) struct WorkspaceContext {
    pub(super) instructions: String,
    pub(super) guideline: Option<String>,
    pub(super) tag_list: TagList,
    pub(super) folders: Vec<FolderInput>,
}

impl WorkspaceContext {
    pub(super) fn load(
        store: &SharedStore,
        id: WorkspaceId,
    ) -> knoa_store::Result<WorkspaceContext> {
        let store = lock(store);
        let workspace = store.workspace(id)?;
        let folders = store
            .folders(id)?
            .into_iter()
            .filter(|f| f.auto_assign)
            .map(|f| FolderInput {
                id: f.id,
                name: f.name,
                description: f.description,
            })
            .collect();
        Ok(WorkspaceContext {
            instructions: workspace
                .as_ref()
                .map(|w| w.instructions.clone())
                .unwrap_or_default(),
            guideline: workspace
                .as_ref()
                .and_then(|w| w.screening.active_guideline())
                .map(str::to_owned),
            tag_list: workspace.map(|w| w.tag_list).unwrap_or_default(),
            folders,
        })
    }
}
