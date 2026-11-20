//! The page's last group: Create for a new workspace, Delete for an existing one.

use gpui::{
    Context, Div, ParentElement as _, StatefulInteractiveElement as _, Styled as _, div,
    prelude::FluentBuilder as _,
};
use knoa_core::WorkspaceId;

use super::{WorkspacePage, form};
use crate::{
    i18n::t,
    ui::{
        components::{Tone, button},
        theme::Theme,
    },
};

impl WorkspacePage {
    /// Create for a new workspace; Delete for an existing one, unless it's the last.
    pub(super) fn footer(&self, theme: &Theme, cx: &mut Context<'_, WorkspacePage>) -> Option<Div> {
        let Some(id) = self.existing else {
            return Some(
                div().flex().justify_end().child(
                    button(
                        "create-workspace",
                        t!("settings-workspace-create"),
                        Tone::Primary,
                        cx,
                    )
                    .on_click(cx.listener(|this, _, window, cx| this.create(window, cx))),
                ),
            );
        };
        if self.library.read(cx).workspaces().len() < 2 {
            return None;
        }
        let label = if self.confirm_delete {
            t!("settings-workspace-delete-confirm")
        } else {
            t!("settings-workspace-delete-button")
        };
        let delete = button("delete-workspace", label, Tone::Soft, cx)
            .when(self.confirm_delete, |s| s.text_color(theme.danger))
            .on_click(cx.listener(move |this, _, _, cx| this.delete(id, cx)));
        let row = form::row(
            &t!("settings-workspace-delete-label"),
            Some(&t!("settings-workspace-delete-about")),
            delete,
            theme,
        );
        Some(form::group(
            &t!("settings-workspace-delete"),
            vec![row],
            theme,
        ))
    }

    fn delete(&mut self, id: WorkspaceId, cx: &mut Context<'_, WorkspacePage>) {
        if !self.confirm_delete {
            self.confirm_delete = true;
            cx.notify();
            return;
        }
        self.deleted = true;
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| store.delete_workspace(id).map(drop));
        });
    }
}
