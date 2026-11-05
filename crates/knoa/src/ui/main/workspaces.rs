//! The workspace switcher, opened from the workspace button in the sidebar's
//! titlebar.

use gpui::{
    Context, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};

use knoa_core::Workspace;

use super::MainView;
use crate::ui::{
    components::{icon, workspace_mark},
    theme::Theme,
};

impl MainView {
    pub(super) fn workspace_switcher(&self, cx: &mut Context<'_, MainView>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let library = self.library.read(cx);
        let current = library.workspace_id();
        let workspaces = library.workspaces().to_vec();

        let mut list = div().flex().flex_col().gap_0p5();
        for workspace in &workspaces {
            list = list.child(self.workspace_row(workspace, workspace.id == current, cx));
        }

        div()
            .w(px(320.))
            .p_3()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .px_2()
                    .pt_1()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.faint)
                    .child("Workspaces"),
            )
            .child(list)
            .child(
                div()
                    .id("new-workspace")
                    .flex()
                    .items_center()
                    .gap_2()
                    .h(px(34.))
                    .px_2()
                    .rounded(px(10.))
                    .text_sm()
                    .text_color(theme.muted)
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.hover))
                    .child(icon("plus", px(15.), theme.muted))
                    .child("New workspace")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.edit_workspace(None, window, cx);
                    })),
            )
    }

    fn workspace_row(
        &self,
        workspace: &Workspace,
        is_current: bool,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let id = workspace.id;
        let source = synced_from(&workspace.browsers);
        div()
            .id(("workspace", id.0 as usize))
            .p_2()
            .flex()
            .items_center()
            .gap_2p5()
            .rounded(px(10.))
            .cursor_pointer()
            .when(is_current, |s| s.bg(theme.soft))
            .hover(|s| s.bg(theme.hover))
            .child(workspace_mark(
                &workspace.name,
                workspace.color,
                px(28.),
                cx,
            ))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .truncate()
                            .child(workspace.name.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted)
                            .truncate()
                            .child(source),
                    ),
            )
            .child(
                div()
                    .id(("edit-workspace", id.0 as usize))
                    .size(px(26.))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .hover(|s| s.bg(theme.panel))
                    .child(icon("pencil", px(14.), theme.muted))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        cx.stop_propagation();
                        this.edit_workspace(Some(id), window, cx);
                    })),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                this.switch_workspace(id, window, cx);
            }))
    }
}

/// "Helium, Chrome", or that links are only added by hand.
fn synced_from(browsers: &[String]) -> String {
    let mut names: Vec<&str> = browsers
        .iter()
        .filter_map(|id| knoa_browser::ProfileId::parse(id))
        .map(|id| id.browser.name())
        .collect();
    names.dedup();
    if names.is_empty() {
        "Links added by hand".to_owned()
    } else {
        format!("From {}", names.join(", "))
    }
}
