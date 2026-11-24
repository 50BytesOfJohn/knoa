//! The corner capsule at the top right of the library, and the Display popover its
//! button opens: how the library looks, kept apart from the search, which is what it
//! shows.

use gpui::{
    Context, InteractiveElement as _, IntoElement, MouseButton, ParentElement as _, Pixels,
    StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use gpui_component::switch::Switch;

use super::{MainView, content::content_span, overlay::Overlay, search::PILL_HEIGHT};
use crate::{
    i18n::t,
    state::settings::SettingsFile,
    ui::{
        components::{icon, tip},
        theme::{MARGIN, Theme},
    },
};

const CAPSULE_HEIGHT: f32 = 36.;
const BUTTON_SIZE: f32 = 30.;
const POPOVER_WIDTH: f32 = 236.;
const POPOVER_GAP: f32 = 8.;
const ROW_HEIGHT: f32 = 34.;

/// The capsule's top, which centres it on the pill.
fn capsule_top() -> Pixels {
    MARGIN + px((PILL_HEIGHT - CAPSULE_HEIGHT) / 2.)
}

/// Where the Display popover hangs: right-aligned with the capsule, just below it. The
/// distance is from the window's right edge.
pub(super) fn popover_anchor(show_detail: bool, window: &Window) -> (Pixels, Pixels) {
    let (_, right) = content_span(window, show_detail);
    let from_right = window.viewport_size().width - px(right);
    (capsule_top() + px(CAPSULE_HEIGHT + POPOVER_GAP), from_right)
}

impl MainView {
    /// Its right edge sits on the content's, so it moves left when the detail panel opens.
    pub(super) fn render_capsule(
        &self,
        show_detail: bool,
        window: &Window,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let (_, right) = content_span(window, show_detail);
        let open = matches!(self.overlay, Some(Overlay::Display));
        div()
            .absolute()
            .top(capsule_top())
            .left(px(right - CAPSULE_HEIGHT))
            .h(px(CAPSULE_HEIGHT))
            .p(px(3.))
            .flex()
            .items_center()
            .rounded_full()
            .bg(theme.panel)
            .border_1()
            .border_color(theme.hairline)
            .shadow_sm()
            .child(
                div()
                    .id("display-button")
                    .size(px(BUTTON_SIZE - 2.))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .bg(if open {
                        theme.hover
                    } else {
                        gpui::transparent_black()
                    })
                    .hover(|s| s.bg(theme.hover))
                    .child(icon("sliders-horizontal", px(15.), theme.muted))
                    .tooltip(tip(t!("main-display"), None, &theme))
                    .on_click(cx.listener(|this, _, window, cx| {
                        if matches!(this.overlay, Some(Overlay::Display)) {
                            this.close_overlay(window, cx);
                        } else {
                            this.open_overlay(Overlay::Display, cx);
                        }
                    })),
            )
    }

    pub(super) fn display_popover(&self, cx: &mut Context<'_, MainView>) -> impl IntoElement {
        let settings = SettingsFile::get(cx).clone();
        let row = |label: String, id: &'static str, on: bool| {
            div()
                .h(px(ROW_HEIGHT))
                .px(px(10.))
                .flex()
                .items_center()
                .gap_3()
                .text_sm()
                .cursor_pointer()
                .child(div().flex_1().child(label))
                .child(Switch::new(id).checked(on))
        };
        div()
            .w(px(POPOVER_WIDTH))
            .p(px(6.))
            .flex()
            .flex_col()
            .child(
                row(
                    t!("main-display-summaries"),
                    "show-summaries",
                    settings.show_summaries,
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        let on = !SettingsFile::get(cx).show_summaries;
                        this.set_show_summaries(on, cx);
                    }),
                ),
            )
            .child(
                row(
                    t!("main-display-up-next"),
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
    }
}
