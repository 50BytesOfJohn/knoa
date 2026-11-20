//! The sidebar's top row: room for the traffic lights, the workspace button and the
//! add-link button.

use gpui::{
    Context, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _, Pixels,
    SharedString, StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _,
    px,
};
use knoa_core::Ink;

use super::{
    LIGHTS_GAP, LIGHTS_INSET, LIGHTS_WIDTH, SidebarEvent, SidebarView, TITLEBAR_HEIGHT,
    WORKSPACE_PADDING,
};
use crate::{
    assets::line_icon,
    i18n::t,
    ui::{
        components::{svg_icon, tip, workspace_mark},
        main::NewLink,
        theme::{ROW_RADIUS, Theme},
    },
};

/// The workspace button and the titlebar's icon buttons.
pub(super) const BUTTON: Pixels = px(28.);
const BUTTON_ICON: Pixels = px(16.);
const WORKSPACE_MARK: Pixels = px(18.);

impl SidebarView {
    pub(super) fn titlebar(&self, cx: &mut Context<'_, SidebarView>) -> impl IntoElement {
        let (name, color) = self
            .library
            .read(cx)
            .workspace()
            .map(|w| (SharedString::from(w.name.clone()), w.color))
            .unwrap_or_default();
        div()
            .h(TITLEBAR_HEIGHT)
            .flex_none()
            .flex()
            .items_center()
            .gap(px(2.))
            .pl(LIGHTS_INSET + LIGHTS_WIDTH + LIGHTS_GAP)
            .pr(px(8.))
            .child(self.workspace_button(name, color, cx))
            .child(div().flex_1())
            .child(add_link_button(cx))
    }

    fn workspace_button(
        &self,
        name: SharedString,
        color: Ink,
        cx: &mut Context<'_, SidebarView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx);
        let mark = workspace_mark(&name, color, WORKSPACE_MARK, cx);
        div()
            .id("workspace-button")
            .min_w_0()
            .h(BUTTON)
            .pl(WORKSPACE_PADDING)
            .pr(px(6.))
            .flex()
            .items_center()
            .gap(px(8.))
            .rounded(ROW_RADIUS)
            .cursor_pointer()
            .when(self.workspaces_open, |s| s.bg(theme.selected_row()))
            .hover(|s| s.bg(theme.hover))
            .on_click(cx.listener(|_, _, _, cx| cx.emit(SidebarEvent::OpenWorkspaces)))
            .child(mark.rounded(px(5.)).text_size(px(10.5)))
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(name),
            )
            .child(svg_icon(
                line_icon!("chevrons-up-down"),
                px(12.),
                theme.muted,
            ))
    }
}

fn add_link_button(cx: &mut Context<'_, SidebarView>) -> impl IntoElement {
    let theme = Theme::get(cx);
    icon_button("add-link", line_icon!("plus"), BUTTON, BUTTON_ICON, theme)
        .tooltip(tip(t!("sidebar-add-link"), Some(&NewLink), theme))
        .on_click(cx.listener(|_, _, _, cx| cx.emit(SidebarEvent::NewLink)))
}

/// A square icon button: `muted` at rest, `text` and the hover fill under the pointer.
pub(super) fn icon_button(
    id: &'static str,
    path: SharedString,
    size: Pixels,
    icon: Pixels,
    theme: &Theme,
) -> gpui::Stateful<gpui::Div> {
    let (hover, text) = (theme.hover, theme.text);
    div()
        .id(id)
        .group(id)
        .flex_none()
        .size(size)
        .rounded(ROW_RADIUS)
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .child(svg_icon(path, icon, theme.muted).group_hover(id, move |s| s.text_color(text)))
}
