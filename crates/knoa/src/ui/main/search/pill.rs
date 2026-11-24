//! The search pill: the search icon, the tokens, the input, the result count and the
//! clear button.

use gpui::{
    BoxShadow, Context, Focusable as _, FontWeight, Hsla, InteractiveElement as _, IntoElement,
    MouseButton, ParentElement as _, StatefulInteractiveElement as _, Styled as _, Window, div,
    point, prelude::FluentBuilder as _, px,
};
use gpui_component::input::Input;
use knoa_core::Place;

use super::{PILL_HEIGHT, tokens::TOKEN_GAP};
use crate::{
    i18n::t,
    ui::{
        components::{icon, text_width},
        main::MainView,
        theme::Theme,
    },
};

/// The pill's padding at each end, and the gap between its parts.
const PADDING: f32 = 16.;
const GAP: f32 = 10.;
const SEARCH_ICON: f32 = 17.;
const CLEAR_SIZE: f32 = 30.;
/// The input never gets narrower than this; tokens fold into "+N" first.
const INPUT_MIN_WIDTH: f32 = 120.;
const COUNT_TEXT_SIZE: f32 = 12.;

impl MainView {
    /// The pill `width` wide, starting at `left` in the window.
    pub(super) fn render_pill(
        &mut self,
        left: f32,
        width: f32,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let focused = self.search.read(cx).focus_handle(cx).is_focused(window);
        let library = self.library.read(cx);
        let searching = !library.search().is_empty();
        let clearable = searching || library.place() != Place::Everything;
        let count =
            searching.then(|| t!("main-search-found", count = (library.result_facets().total)));

        let mut fixed = PADDING * 2. + SEARCH_ICON + GAP + INPUT_MIN_WIDTH;
        if let Some(count) = &count {
            fixed += GAP
                + f32::from(text_width(
                    count,
                    px(COUNT_TEXT_SIZE),
                    FontWeight::NORMAL,
                    window,
                ));
        }
        if clearable {
            fixed += GAP + CLEAR_SIZE;
        }
        let tokens_left = left + 1. + PADDING + SEARCH_ICON + GAP;
        let tokens = self.render_tokens(tokens_left, width - fixed - GAP, window, cx);

        let pill = div().id("search-pill").key_context("SearchBar");
        self.with_pill_keys(pill, cx)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    this.pill.selected = None;
                    this.focus_pill(window, cx);
                }),
            )
            .w(px(width))
            .h(px(PILL_HEIGHT))
            .px(px(PADDING))
            .flex()
            .items_center()
            .gap(px(GAP))
            .rounded_full()
            .bg(theme.panel)
            .border_1()
            .map(|s| {
                if focused {
                    s.border_color(theme.hairline.blend(theme.accent.opacity(0.55)))
                        .shadow(focus_ring(theme.accent))
                } else {
                    s.border_color(theme.hairline).shadow_md()
                }
            })
            .child(icon("search", px(SEARCH_ICON), theme.muted))
            .when(!tokens.is_empty(), |s| {
                s.child(
                    div()
                        .flex_none()
                        .flex()
                        .items_center()
                        .gap(px(TOKEN_GAP))
                        .children(tokens),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_w(px(INPUT_MIN_WIDTH))
                    .child(Input::new(&self.search).appearance(false).cleanable(false)),
            )
            .when_some(count, |s, count| {
                s.child(
                    div()
                        .flex_none()
                        .text_size(px(COUNT_TEXT_SIZE))
                        .text_color(theme.muted)
                        .child(count),
                )
            })
            .when(clearable, |s| s.child(self.clear_button(&theme, cx)))
    }

    fn clear_button(&self, theme: &Theme, cx: &mut Context<'_, MainView>) -> impl IntoElement {
        div()
            .id("search-clear")
            .flex_none()
            .size(px(CLEAR_SIZE))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.bg(theme.hover))
            .child(icon("x", px(15.), theme.muted))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(|this, _, window, cx| this.clear_all(window, cx)))
    }
}

/// The focused pill's soft accent ring, over the resting shadow.
fn focus_ring(accent: Hsla) -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            color: accent.opacity(0.18),
            offset: point(px(0.), px(0.)),
            blur_radius: px(0.),
            spread_radius: px(3.),
            inset: false,
        },
        BoxShadow {
            color: gpui::black().opacity(0.08),
            offset: point(px(0.), px(4.)),
            blur_radius: px(8.),
            spread_radius: px(-2.),
            inset: false,
        },
    ]
}
