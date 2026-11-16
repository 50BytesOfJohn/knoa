//! What the bookmark's workspace tells the AI.

use knoa_ai::{FolderAncestor, FolderInput};
use knoa_core::{TagList, WorkspaceId};
use knoa_store::Store;

use crate::state::database::Database;

/// The workspace's instructions, guideline, tag list and the folders it may file into.
pub(super) struct WorkspaceContext {
    pub(super) instructions: String,
    pub(super) guideline: Option<String>,
    pub(super) tag_list: TagList,
    pub(super) folders: Vec<FolderInput>,
}

impl WorkspaceContext {
    pub(super) fn load(
        database: &Database,
        id: WorkspaceId,
    ) -> knoa_store::Result<WorkspaceContext> {
        database
            .read()
            .snapshot(|store| WorkspaceContext::read(store, id))
    }

    fn read(store: &Store, id: WorkspaceId) -> knoa_store::Result<WorkspaceContext> {
        let workspace = store.workspace(id)?;
        let tree = store.folders(id)?;
        // The full path and the parents' descriptions, so "Anime" under "Wallpapers" is
        // matched as wallpapers. Parents are listed whether or not they auto-assign.
        let folders = tree
            .folders()
            .iter()
            .filter(|f| f.auto_assign)
            .map(|f| {
                let path = tree.path(f.id);
                let ancestors = path[..path.len().saturating_sub(1)]
                    .iter()
                    .map(|a| FolderAncestor {
                        id: a.id,
                        name: tree.path_name(a.id),
                        description: a.description.clone(),
                    })
                    .collect();
                FolderInput {
                    id: f.id,
                    name: tree.path_name(f.id),
                    description: f.description.clone(),
                    ancestors,
                }
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
