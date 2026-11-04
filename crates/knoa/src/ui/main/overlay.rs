//! Floating panels over the main screen: settings, view options, the workspace switcher,
//! the folder and workspace editors, and adding a link.
//! Clicking outside closes them.

use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, Styled as _, Window, deferred, div, px,
};
use gpui_component::switch::Switch;

use super::{MainView, Overlay};
use crate::{
    state::settings::SettingsFile,
    ui::theme::{PANEL_RADIUS, Theme},
};

impl MainView {
    pub(super) fn render_overlay(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> Option<AnyElement> {
        let theme = Theme::get(cx).clone();
        let panel: AnyElement = match self.overlay.as_ref()? {
            Overlay::Settings(panel) => panel.clone().into_any_element(),
            Overlay::Folder(editor) => editor.clone().into_any_element(),
            Overlay::Workspace(editor) => editor.clone().into_any_element(),
            Overlay::AddLink(panel) => panel.clone().into_any_element(),
            Overlay::ViewOptions => self.view_options(cx).into_any_element(),
            Overlay::Workspaces => self.workspace_switcher(cx).into_any_element(),
        };
        let scrim = div()
            .id("scrim")
            .absolute()
            .inset_0()
            .bg(gpui::black().opacity(if theme.dark { 0.35 } else { 0.12 }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| this.close_overlay(window, cx)),
            )
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .id("overlay-panel")
                    .occlude()
                    .rounded(PANEL_RADIUS)
                    .bg(theme.panel)
                    .border_1()
                    .border_color(theme.hairline)
                    .shadow_lg()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(panel),
            );
        Some(deferred(scrim).with_priority(1).into_any_element())
    }

    fn view_options(&self, cx: &mut Context<'_, MainView>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let settings = SettingsFile::get(cx).clone();
        let row = |label: &'static str, id: &'static str, on: bool| {
            div()
                .flex()
                .items_center()
                .gap_3()
                .text_sm()
                .child(div().flex_1().child(label))
                .child(Switch::new(id).checked(on))
        };
        div()
            .w(px(300.))
            .p_5()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.faint)
                    .child("View"),
            )
            .child(
                row(
                    "Show summaries on cards",
                    "show-summaries",
                    settings.show_summaries,
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        SettingsFile::update(cx, |s| s.show_summaries = !s.show_summaries);
                        this.layout = super::content::LayoutCache::default();
                        cx.notify();
                    }),
                ),
            )
            .child(
                row(
                    "Show the Up next strip",
                    "show-up-next",
                    settings.show_up_next,
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|_, _, _, cx| {
                        SettingsFile::update(cx, |s| s.show_up_next = !s.show_up_next);
                        cx.notify();
                    }),
                ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted)
                    .child("Cards are grouped by date. Searching ranks by best match."),
            )
    }
}
