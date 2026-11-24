//! The tokens in the pill: the scope first, then the search's tokens, the oldest folded
//! into "+N" when they would crowd the input.

use gpui::{
    AnyElement, Context, Div, FontWeight, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};

use super::{
    MainView, Slot,
    labels::{Chip, scope_chip, token_chip},
};
use crate::ui::{
    components::{icon, text_width},
    main::overlay::Overlay,
    theme::Theme,
};

const HEIGHT: f32 = 26.;
const PADDING_LEFT: f32 = 8.;
const PADDING_RIGHT: f32 = 6.;
const INNER_GAP: f32 = 5.;
const ICON: f32 = 12.;
const END_ICON: f32 = 11.;
const LABEL_SIZE: f32 = 12.5;
/// A token's padding, mark, gaps, end icon and border: all of it but the label.
const CHROME: f32 = PADDING_LEFT + ICON + INNER_GAP + INNER_GAP + END_ICON + PADDING_RIGHT + 2.;
/// Between tokens, as the pill lays them out.
pub(super) const TOKEN_GAP: f32 = 6.;

/// The scope's label is cut short past this, and further, down to the minimum, when
/// even the scope beside "+N" doesn't fit.
const SCOPE_LABEL_MAX: f32 = 180.;
const SCOPE_LABEL_MIN: f32 = 60.;

/// The tokens as drawn in the pill, and the width they take from the first to the last.
pub(super) struct Tokens {
    pub chips: Vec<AnyElement>,
    pub width: f32,
}

impl MainView {
    /// The tokens that fit in `room`, with "+N" for the oldest ones that don't. `left` is
    /// where the first token starts in the window.
    pub(super) fn render_tokens(
        &mut self,
        left: f32,
        room: f32,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> Tokens {
        let theme = Theme::get(cx).clone();
        let library = self.library.read(cx);
        let scope = scope_chip(library, &theme);
        let tokens: Vec<Chip> = library
            .search()
            .tokens
            .iter()
            .enumerate()
            .map(|(index, token)| token_chip(index, token))
            .collect();
        let widths: Vec<f32> = tokens
            .iter()
            .map(|chip| CHROME + label_width(&chip.label, window))
            .collect();
        let scope_label = scope.as_ref().map(|chip| label_width(&chip.label, window));
        // `fit` counts a gap after every token, the last one too.
        let layout = fit(scope_label, &widths, room + TOKEN_GAP, |n| {
            CHROME + label_width(&folded_label(n), window)
        });
        self.pill.folded = layout.folded;

        let selected = self.pill.selected;
        let mut chips = Vec::new();
        if let Some(scope) = scope {
            let label_max = Some(layout.scope_label);
            chips.push(self.token(&scope, label_max, selected == Some(scope.slot), &theme, cx));
        }
        if layout.folded > 0 {
            let left = left + layout.scope_width();
            chips.push(self.folded_token(layout.folded, left, &theme, cx));
        }
        for chip in tokens.iter().skip(layout.folded) {
            chips.push(self.token(chip, None, selected == Some(chip.slot), &theme, cx));
        }
        Tokens {
            chips,
            width: (layout.width - TOKEN_GAP).max(0.),
        }
    }

    /// A token, its label cut short with an ellipsis past `label_max`.
    fn token(
        &self,
        chip: &Chip,
        label_max: Option<f32>,
        selected: bool,
        theme: &Theme,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        let slot = chip.slot;
        let mark = chip.mark.render(px(ICON), theme);
        let remove = div()
            .id(("token-remove", slot_id(slot)))
            .flex_none()
            .rounded_full()
            .cursor_pointer()
            .text_color(theme.muted)
            .hover(|s| s.text_color(theme.text))
            .child(icon("x", px(END_ICON), theme.muted).hover(|s| s.text_color(theme.text)))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(move |this, _, window, cx| {
                this.remove_token(slot, window, cx);
            }));
        frame(("token", slot_id(slot)), theme)
            .map(|s| style_for(s, slot, selected, theme))
            .child(mark)
            .child(
                label(chip.label.clone())
                    .when_some(label_max, |s, max| s.max_w(px(max)).truncate()),
            )
            .child(remove)
            .into_any_element()
    }

    fn folded_token(
        &self,
        count: usize,
        left: f32,
        theme: &Theme,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        frame("token-folded", theme)
            .bg(theme.soft)
            .cursor_pointer()
            .child(label(folded_label(count).into()))
            .child(icon("chevron-down", px(END_ICON), theme.muted))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(move |this, _, _, cx| {
                this.open_overlay(Overlay::FoldedTokens { left: px(left) }, cx);
            }))
            .into_any_element()
    }
}

/// Where the tokens go in the pill: how wide the scope's label is drawn, how many of
/// the oldest tokens fold into "+N", and the width all of it takes.
struct Layout {
    scope_label: f32,
    has_scope: bool,
    folded: usize,
    width: f32,
}

impl Layout {
    fn scope_width(&self) -> f32 {
        if self.has_scope {
            CHROME + self.scope_label + TOKEN_GAP
        } else {
            0.
        }
    }
}

/// Folds the fewest of the oldest tokens so the rest fit in `room` after the scope.
/// When even the scope beside "+N" with every token doesn't fit, the scope's label gives
/// way, down to its minimum; past that the tokens take more than `room`.
fn fit(
    scope_label: Option<f32>,
    widths: &[f32],
    room: f32,
    folded_width: impl Fn(usize) -> f32,
) -> Layout {
    let mut layout = Layout {
        scope_label: scope_label.unwrap_or_default().min(SCOPE_LABEL_MAX),
        has_scope: scope_label.is_some(),
        folded: 0,
        width: 0.,
    };
    let after_scope = |n: usize| -> f32 {
        let rest: f32 = widths[n..].iter().map(|w| w + TOKEN_GAP).sum();
        let folded = if n > 0 {
            folded_width(n) + TOKEN_GAP
        } else {
            0.
        };
        folded + rest
    };
    let scope = layout.scope_width();
    if let Some(n) = (0..=widths.len()).find(|&n| scope + after_scope(n) <= room) {
        layout.folded = n;
    } else {
        layout.folded = widths.len();
        if layout.has_scope {
            let spare = room - after_scope(widths.len()) - CHROME - TOKEN_GAP;
            let min = SCOPE_LABEL_MIN.min(layout.scope_label);
            layout.scope_label = spare.clamp(min, layout.scope_label);
        }
    }
    layout.width = layout.scope_width() + after_scope(layout.folded);
    layout
}

fn folded_label(count: usize) -> String {
    format!("+{count}")
}

fn label_width(label: &str, window: &Window) -> f32 {
    f32::from(text_width(
        label,
        px(LABEL_SIZE),
        FontWeight::MEDIUM,
        window,
    ))
}

fn frame(id: impl Into<gpui::ElementId>, theme: &Theme) -> gpui::Stateful<Div> {
    div()
        .id(id)
        .flex_none()
        .h(px(HEIGHT))
        .pl(px(PADDING_LEFT))
        .pr(px(PADDING_RIGHT))
        .flex()
        .items_center()
        .gap(px(INNER_GAP))
        .rounded_full()
        .border_1()
        .border_color(gpui::transparent_black())
        .text_color(theme.text)
}

/// The scope reads as "where" rather than "what": an outline, no fill.
fn style_for(
    s: gpui::Stateful<Div>,
    slot: Slot,
    selected: bool,
    theme: &Theme,
) -> gpui::Stateful<Div> {
    let s = match slot {
        Slot::Scope => s.border_color(theme.hairline),
        Slot::Token(_) => s.bg(theme.soft),
    };
    if selected {
        s.border_color(theme.accent)
    } else {
        s
    }
}

fn label(text: SharedString) -> Div {
    div()
        .text_size(px(LABEL_SIZE))
        .font_weight(FontWeight::MEDIUM)
        .whitespace_nowrap()
        .child(text)
}

pub(super) fn slot_id(slot: Slot) -> usize {
    match slot {
        Slot::Scope => usize::MAX,
        Slot::Token(index) => index,
    }
}
