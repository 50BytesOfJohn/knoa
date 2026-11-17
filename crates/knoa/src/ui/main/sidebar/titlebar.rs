//! The sidebar's top row: room for the traffic lights, then the workspace and add-link
//! buttons.

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};
use gpui_component::tooltip::Tooltip;
use knoa_core::Ink;

use super::{
    BUTTON_ICON, BUTTON_PADDING, SidebarEvent, SidebarView, TITLEBAR_BUTTON, TITLEBAR_GAP,
    TITLEBAR_HEIGHT, WORKSPACE_BUTTON_WIDTH, WORKSPACE_MARK,
};
use crate::ui::{
    components::{icon, workspace_mark},
    theme::{ROW_RADIUS, Theme},
};

impl SidebarView {
    pub(super) fn titlebar(
        &self,
        (name, color): (String, Ink),
        cx: &mut Context<'_, SidebarView>,
    ) -> impl IntoElement {
        div()
            .h(TITLEBAR_HEIGHT)
            .flex_none()
            .flex()
            .items_center()
            .justify_end()
            .gap(TITLEBAR_GAP)
            .pr(BUTTON_PADDING)
            .child(self.workspace_button(&name, color, cx))
            .child(add_link_button(cx))
    }

    /// Only the workspace's mark: most people keep one workspace, so its name would say
    /// little. The name shows on hover and in the switcher.
    fn workspace_button(
        &self,
        name: &str,
        color: Ink,
        cx: &mut Context<'_, SidebarView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx);
        let tooltip = SharedString::from(name.to_owned());
        let (bg, fg, border) = (theme.panel, theme.text, theme.hairline);
        div()
            .id("workspace-button")
            .w(WORKSPACE_BUTTON_WIDTH)
            .h(TITLEBAR_BUTTON)
            .pl_1()
            .flex()
            .items_center()
            .gap(px(3.))
            .rounded(ROW_RADIUS)
            .cursor_pointer()
            .when(self.workspaces_open, |s| s.bg(theme.hover))
            .hover(|s| s.bg(theme.hover))
            .tooltip(move |window, cx| {
                Tooltip::new(tooltip.clone())
                    .bg(bg)
                    .text_color(fg)
                    .border_color(border)
                    .build(window, cx)
            })
            .on_click(cx.listener(|_, _, _, cx| cx.emit(SidebarEvent::OpenWorkspaces)))
            .child(workspace_mark(name, color, WORKSPACE_MARK, cx))
            .child(icon("chevron-down", px(10.), theme.faint))
    }
}

fn add_link_button(cx: &mut Context<'_, SidebarView>) -> impl IntoElement {
    let theme = Theme::get(cx);
    div()
        .id("add-link")
        .size(TITLEBAR_BUTTON)
        .rounded(ROW_RADIUS)
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(|s| s.bg(theme.hover))
        .child(icon("plus", BUTTON_ICON, theme.muted))
        .on_click(cx.listener(|_, _, _, cx| cx.emit(SidebarEvent::NewLink)))
}
