//! The search pill: the search icon, the tokens, the input, the result count and the
//! clear button.

use gpui::{
    Animation, AnimationExt as _, AnyElement, BoxShadow, Context, Focusable as _, FontWeight, Hsla,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, Window, div, point, prelude::FluentBuilder as _,
    px,
};
use gpui_component::input::Input;
use knoa_core::Place;

use super::{PILL_HEIGHT, tokens::TOKEN_GAP};
use crate::{
    i18n::t,
    ui::{
        components::{icon, text_width},
        main::MainView,
        motion,
        theme::Theme,
    },
};

/// The pill's padding at its right end, and at its left beside the search icon.
const PADDING: f32 = 16.;
/// The pill's padding at its left end, where the first token starts once the icon is
/// gone. Less than [`PADDING`]: a token's round end sits closer to the pill's.
const TOKENS_PADDING: f32 = 9.;
/// Between the icon and what follows it, and before the count and the clear button.
const GAP: f32 = 10.;
const SEARCH_ICON: f32 = 17.;
/// The icon with the space around it, past [`TOKENS_PADDING`].
const ICON_SLOT: f32 = PADDING - TOKENS_PADDING + SEARCH_ICON + GAP;
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
        let scoped = library.place() != Place::Everything;
        let has_tokens = scoped || !library.search().tokens.is_empty();
        let count =
            searching.then(|| t!("main-search-found", count = (library.result_facets().total)));

        // What comes before the tokens: the border, the padding and, without tokens, the
        // icon.
        let lead = 1. + TOKENS_PADDING + if has_tokens { 0. } else { ICON_SLOT };
        // Everything but the tokens and the input, with the gaps after each.
        let mut fixed = lead + PADDING + 1.;
        if let Some(count) = &count {
            fixed += GAP
                + f32::from(text_width(
                    count,
                    px(COUNT_TEXT_SIZE),
                    FontWeight::NORMAL,
                    window,
                ));
        }
        if searching || scoped {
            fixed += GAP + CLEAR_SIZE;
        }
        let room = width - fixed - INPUT_MIN_WIDTH - TOKEN_GAP;
        let tokens = self.render_tokens(left + lead, room, window, cx);
        let tokens_width = if tokens.chips.is_empty() {
            0.
        } else {
            tokens.width + TOKEN_GAP
        };
        // Tokens that can't give way any further take from the input instead of pushing
        // the count and the clear button out of the pill.
        let input_min = (width - fixed - tokens_width).clamp(0., INPUT_MIN_WIDTH);
        let icon = self.search_icon(!has_tokens, &theme);

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
            .pl(px(TOKENS_PADDING))
            .pr(px(PADDING))
            .flex()
            .items_center()
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
            .child(icon)
            .when(!tokens.chips.is_empty(), |s| {
                s.child(
                    div()
                        .flex_none()
                        .mr(px(TOKEN_GAP))
                        .flex()
                        .items_center()
                        .gap(px(TOKEN_GAP))
                        .children(tokens.chips),
                )
            })
            .child(
                div().flex_1().min_w(px(input_min)).child(
                    Input::new(&self.search)
                        .appearance(false)
                        .cleanable(false)
                        .px_0(),
                ),
            )
            .when_some(count, |s, count| {
                s.child(
                    div()
                        .flex_none()
                        .ml(px(GAP))
                        .text_size(px(COUNT_TEXT_SIZE))
                        .text_color(theme.muted)
                        .child(count),
                )
            })
            .when(searching || scoped, |s| {
                s.child(self.clear_button(&theme, cx))
            })
    }

    /// The search icon in its slot, there while the pill holds no token. It fades and its
    /// slot narrows as it goes, and the other way as it comes back.
    fn search_icon(&mut self, shown: bool, theme: &Theme) -> AnyElement {
        if self.pill.icon_hidden == shown {
            self.pill.icon_hidden = !shown;
            self.pill.icon_turns += 1;
        }
        let muted = theme.muted;
        let slot = move |presence: f32| {
            div()
                .flex_none()
                .overflow_hidden()
                .w(px(ICON_SLOT * presence))
                .opacity(presence)
                // The padding goes on the icon, not the slot: the slot's own padding
                // would keep it from narrowing to nothing.
                .child(
                    div()
                        .pl(px(PADDING - TOKENS_PADDING))
                        .child(icon("search", px(SEARCH_ICON), muted)),
                )
        };
        let end = if shown { 1. } else { 0. };
        if self.pill.icon_turns == 0 {
            return slot(end).into_any_element();
        }
        slot(end)
            .with_animation(
                ("search-icon", self.pill.icon_turns),
                Animation::new(motion::MENU_OPEN),
                move |_, delta| slot((1. - end) + (2. * end - 1.) * delta),
            )
            .into_any_element()
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
