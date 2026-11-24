//! The search at the top of the library: the pill with its tokens, and the kind chips
//! under it. Every change to the search goes through [`MainView::set_search`].

mod folded;
mod keys;
mod labels;
mod pill;
mod tokens;

use gpui::{
    Context, Entity, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_component::input::{InputEvent, InputState};
use knoa_core::{Filter, Kind, Place};

use super::MainView;
use crate::{
    i18n::{self, t},
    state::search::{Search, Token},
    ui::theme::{MARGIN, Theme},
};

pub const PILL_HEIGHT: f32 = 50.;
const PILL_MAX_WIDTH: f32 = 620.;
/// The pill, the chips under it and the space before the first card.
pub const SEARCH_AREA_HEIGHT: f32 = PILL_HEIGHT + 52.;
/// What the corner capsule takes at the content's right edge, with the space kept
/// between it and the pill. Kept free on both sides so the pill stays centred.
const CAPSULE_ROOM: f32 = 36. + 16.;

/// A token in the pill: the scope (the place) or one of the search's tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Slot {
    Scope,
    Token(usize),
}

/// What the pill remembers between frames.
#[derive(Default)]
pub(super) struct PillState {
    /// The token the first ⌫ on an empty input selected; the next ⌫ removes it.
    selected: Option<Slot>,
    /// How many of the oldest tokens fold into "+N", as last drawn.
    folded: usize,
}

impl MainView {
    pub(super) fn render_search(
        &mut self,
        show_detail: bool,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let (left, right) = super::content::content_span(window, show_detail);
        let width = pill_width(right - left);
        div()
            .absolute()
            .top(MARGIN)
            .left(px(left))
            .w(px(right - left))
            .flex()
            .flex_col()
            .items_center()
            .gap_2()
            .child(self.render_pill(left + (right - left - width) / 2., width, window, cx))
            .child(self.kind_chips(cx))
    }

    /// After a language switch, what inputs keep rather than draw: their placeholders.
    /// Tokens draw their labels each frame.
    pub(super) fn translate(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        self.sync_placeholder(window, cx);
        if let Some(edits) = &self.tag_edits {
            edits.translate(window, cx);
        }
        if let Some(edit) = &self.heading_edit {
            edit.translate(window, cx);
        }
    }

    /// The one way the UI changes the search: the input shows its typed text, the
    /// placeholder shows only without tokens, and the cards scroll back to the top.
    pub(super) fn set_search(
        &mut self,
        search: Search,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        if self.search.read(cx).value() != search.typed.as_str() {
            let typed = search.typed.clone();
            self.search
                .update(cx, |input, cx| input.set_value(typed, window, cx));
        }
        self.pill.selected = None;
        self.library
            .update(cx, |library, cx| library.set_search(search, cx));
        self.sync_placeholder(window, cx);
        self.scroll.set_offset(gpui::point(px(0.), px(0.)));
        cx.notify();
    }

    pub(super) fn on_search_event(
        &mut self,
        _: &Entity<InputState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        match event {
            InputEvent::Change => self.on_search_input(window, cx),
            InputEvent::PressEnter { .. } => self.lock_typed_text(window, cx),
            // The pill draws its focus ring.
            InputEvent::Focus | InputEvent::Blur => cx.notify(),
        }
    }

    /// Turns the typed text into a text token.
    fn lock_typed_text(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        let mut search = self.library.read(cx).search().clone();
        let text = search
            .typed
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if text.is_empty() {
            return;
        }
        search.typed.clear();
        search.add(Token::Text(text));
        self.set_search(search, window, cx);
    }

    /// Reads what was typed into the search, turning complete syntax words into tokens.
    fn on_search_input(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        let text = self.search.read(cx).value().to_string();
        let mut search = self.library.read(cx).search().clone();
        search.take_input(&text);
        self.set_search(search, window, cx);
    }

    /// Clears the typed text and every token; the place stays.
    pub(super) fn clear_search(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        self.set_search(Search::default(), window, cx);
    }

    /// The pill's ×: the text, the tokens and the scope.
    fn clear_all(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        self.clear_search(window, cx);
        self.show_place(Place::Everything, cx);
        self.focus_pill(window, cx);
    }

    /// Takes a token out of the search. The scope goes by showing Everything, so the
    /// sidebar follows.
    fn remove_token(&mut self, slot: Slot, window: &mut Window, cx: &mut Context<'_, MainView>) {
        match slot {
            Slot::Scope => {
                self.pill.selected = None;
                self.show_place(Place::Everything, cx);
            }
            Slot::Token(index) => {
                let mut search = self.library.read(cx).search().clone();
                if index < search.tokens.len() {
                    search.tokens.remove(index);
                    self.set_search(search, window, cx);
                }
            }
        }
    }

    pub(super) fn focus_pill(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        self.search.update(cx, |input, cx| input.focus(window, cx));
    }

    fn sync_placeholder(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        let placeholder = if self.library.read(cx).search().tokens.is_empty() {
            t!("main-search-placeholder")
        } else {
            String::new()
        };
        self.search.update(cx, |input, cx| {
            input.set_placeholder(placeholder, window, cx);
        });
    }

    /// Adds a kind token, or takes it away when it is the kind already applied.
    fn toggle_kind(&mut self, kind: Kind, window: &mut Window, cx: &mut Context<'_, MainView>) {
        let mut search = self.library.read(cx).search().clone();
        if search.kind() == Some(kind) {
            search
                .tokens
                .retain(|t| *t != Token::Filter(Filter::Kind(kind)));
        } else {
            search.add(Token::Filter(Filter::Kind(kind)));
        }
        self.set_search(search, window, cx);
    }

    fn kind_chips(&self, cx: &mut Context<'_, MainView>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let library = self.library.read(cx);
        let current = library.search().kind();
        let counts = library.base_facets().kinds.clone();
        let chip = |id: &'static str, label: String, active: bool| {
            div()
                .id(id)
                .h(px(28.))
                .px_3()
                .flex()
                .items_center()
                .rounded_full()
                .text_xs()
                .cursor_pointer()
                .font_weight(FontWeight::MEDIUM)
                .map(|s| {
                    if active {
                        s.bg(theme.text).text_color(theme.panel)
                    } else {
                        s.bg(theme.panel.opacity(0.7))
                            .text_color(theme.muted)
                            .hover(|s| s.bg(theme.panel))
                    }
                })
                .child(label)
        };
        let mut row = div().flex().flex_wrap().justify_center().gap_1p5();
        // The Gallery holds media kinds only, so choosing among them says little.
        let in_gallery = library.place() == Place::Gallery;
        if !in_gallery && (counts.len() > 1 || current.is_some()) {
            row = row.child(
                chip("kind-all", t!("main-kind-all"), current.is_none()).on_click(cx.listener(
                    move |this, _, window, cx| {
                        if let Some(kind) = current {
                            this.toggle_kind(kind, window, cx);
                        }
                    },
                )),
            );
            for (kind, count) in counts {
                row = row.child(
                    chip(
                        kind.slug(),
                        t!(
                            "main-kind-chip",
                            kind = (i18n::kind_plural(kind)),
                            count = count
                        ),
                        current == Some(kind),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.toggle_kind(kind, window, cx);
                    })),
                );
            }
        }
        row
    }
}

/// Centred in the content span, never reaching the corner capsule.
fn pill_width(span: f32) -> f32 {
    PILL_MAX_WIDTH.min(span - 2. * CAPSULE_ROOM).max(240.)
}
