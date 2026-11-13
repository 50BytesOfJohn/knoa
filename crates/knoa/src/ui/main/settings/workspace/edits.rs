//! Saving field-level workspace intent and refreshing fields the user left alone.

use gpui::{AppContext as _, Context, Window};
use knoa_core::{
    Screening, Workspace, WorkspaceEdits, WorkspaceId,
    protocol::max_chars::{self, fits},
};

use super::WorkspacePage;
use crate::state::library::Library;

impl WorkspacePage {
    /// Stores only edits relative to this page's baseline.
    pub fn save(&mut self, cx: &mut Context<'_, Self>) {
        let Some(id) = self.existing.filter(|_| !self.deleted) else {
            return;
        };
        let mut workspace = self.workspace(id, cx);
        keep_stored_where_refused(&mut workspace, &self.baseline);
        let edits = WorkspaceEdits::between(&self.baseline, &workspace);
        if edits.is_empty() {
            return;
        }
        let baseline = self.baseline.clone();
        edits.apply_to(&mut self.baseline);
        self.pending_saves += 1;
        let previous = self.saves.take();
        let database = self.library.read(cx).database();
        let library = self.library.clone();
        // Keep saves ordered and let the last one finish even when the sheet closes.
        self.saves = Some(cx.spawn(async move |this, cx| {
            if let Some(previous) = previous {
                previous.await;
            }
            let saved = cx
                .background_spawn(async move { database.write().edit_workspace(id, &edits) })
                .await;
            let _ = this.update(cx, |this, _| {
                this.pending_saves -= 1;
                if let Err(err) = saved {
                    tracing::error!("saving workspace edits failed: {err}");
                    let later = WorkspaceEdits::between(&workspace, &this.baseline);
                    this.baseline = baseline;
                    later.apply_to(&mut this.baseline);
                }
            });
            library.update(cx, Library::refresh);
        }));
    }

    pub fn refresh(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        if self.pending_saves > 0 {
            return;
        }
        let Some(id) = self.existing else {
            return;
        };
        let Some(stored) = self.stored(id, cx) else {
            return;
        };
        let mut current = stored.normalized();
        let edits = WorkspaceEdits::between(&self.baseline, &self.workspace(id, cx));
        self.baseline = current.clone();
        edits.apply_to(&mut current);
        // Trimmed, so a refresh keeps the space or newline the user is still typing.
        if self.name.read(cx).value().trim() != current.name {
            self.name
                .update(cx, |input, cx| input.set_value(current.name, window, cx));
        }
        if self.instructions.read(cx).value().trim() != current.instructions {
            self.instructions.update(cx, |input, cx| {
                input.set_value(current.instructions, window, cx);
            });
        }
        if self.guideline.read(cx).value().trim() != current.screening.guideline {
            self.guideline.update(cx, |input, cx| {
                input.set_value(current.screening.guideline, window, cx);
            });
        }
        self.color = current.color;
        self.screening = current.screening.enabled;
        self.browsers = current.browsers;
        self.opens_in = current.opens_in;
        self.tag_list.update(cx, |editor, cx| {
            editor.refresh(&current.tag_list, window, cx);
        });
        cx.notify();
    }

    pub(super) fn workspace(&self, id: WorkspaceId, cx: &Context<'_, WorkspacePage>) -> Workspace {
        Workspace {
            id,
            name: self.name.read(cx).value().trim().to_owned(),
            color: self.color,
            browsers: self.browsers.clone(),
            opens_in: self.opens_in.clone(),
            instructions: self.instructions.read(cx).value().trim().to_owned(),
            screening: Screening {
                enabled: self.screening,
                guideline: self.guideline.read(cx).value().trim().to_owned(),
            },
            tag_list: self.tag_list.read(cx).tag_list(cx),
        }
    }
}

/// Puts back the stored value of a text field the server would refuse: an emptied name,
/// or text past its limit, which would keep the whole change on this Mac only.
pub(super) fn keep_stored_where_refused(workspace: &mut Workspace, stored: &Workspace) {
    use max_chars::workspace::{GUIDELINE, INSTRUCTIONS, NAME};
    if workspace.name.is_empty() || !fits(&workspace.name, NAME) {
        workspace.name.clone_from(&stored.name);
    }
    if !fits(&workspace.instructions, INSTRUCTIONS) {
        workspace.instructions.clone_from(&stored.instructions);
    }
    if !fits(&workspace.screening.guideline, GUIDELINE) {
        workspace
            .screening
            .guideline
            .clone_from(&stored.screening.guideline);
    }
}
