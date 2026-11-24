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
/// Between tokens, as the pill lays them out.
pub(super) const TOKEN_GAP: f32 = 6.;

impl MainView {
    /// The tokens that fit in `room`, with "+N" for the oldest ones that don't. `left` is
    /// where the first token starts in the window.
    pub(super) fn render_tokens(
        &mut self,
        left: f32,
        room: f32,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> Vec<AnyElement> {
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
        let width = |chip: &Chip| chip_width(&chip.label, window);
        let scope_width = scope.as_ref().map_or(0., |chip| width(chip) + TOKEN_GAP);
        let widths: Vec<f32> = tokens.iter().map(width).collect();
        let folded = folded_count(scope_width, &widths, room, |n| {
            chip_width(&folded_label(n), window)
        });
        self.pill.folded = folded;

        let selected = self.pill.selected;
        let mut chips = Vec::new();
        if let Some(scope) = scope {
            chips.push(self.token(&scope, selected == Some(scope.slot), &theme, cx));
        }
        if folded > 0 {
            let left = left + scope_width;
            chips.push(self.folded_token(folded, left, &theme, cx));
        }
        for chip in tokens.iter().skip(folded) {
            chips.push(self.token(chip, selected == Some(chip.slot), &theme, cx));
        }
        chips
    }

    fn token(
        &self,
        chip: &Chip,
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
            .child(label(chip.label.clone()))
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

/// How many of the oldest tokens fold into "+N" so the rest fit in `room` after the
/// scope: none when all fit, every one when even the last doesn't.
fn folded_count(
    scope: f32,
    widths: &[f32],
    room: f32,
    folded_width: impl Fn(usize) -> f32,
) -> usize {
    let shown = |n: usize| -> f32 {
        let rest: f32 = widths[n..].iter().map(|w| w + TOKEN_GAP).sum();
        let folded = if n > 0 {
            folded_width(n) + TOKEN_GAP
        } else {
            0.
        };
        scope + folded + rest
    };
    (0..=widths.len())
        .find(|&n| shown(n) <= room)
        .unwrap_or(widths.len())
}

fn folded_label(count: usize) -> String {
    format!("+{count}")
}

/// A token's width before it is drawn: its padding, mark, label and end icon.
fn chip_width(label: &str, window: &Window) -> f32 {
    let label = f32::from(text_width(
        label,
        px(LABEL_SIZE),
        FontWeight::MEDIUM,
        window,
    ));
    PADDING_LEFT + ICON + INNER_GAP + label + INNER_GAP + END_ICON + PADDING_RIGHT + 2.
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
