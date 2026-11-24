//! The search at the top of the library: the pill with its tokens and suggestions, and
//! the refine row under it. Every change to the search goes through [`MainView::set_search`].

mod choose;
mod derived;
mod folded;
mod keys;
mod labels;
mod pill;
mod refine;
mod suggest;
mod suggestions;
mod tokens;

use std::rc::Rc;

use derived::Derived;
use gpui::{Context, Entity, IntoElement, ParentElement as _, Styled as _, Window, div, px};
use gpui_component::input::{InputEvent, InputState};
use knoa_core::Place;

use super::{MainView, overlay::Overlay};
use crate::{i18n::t, state::search::Search, ui::theme::MARGIN};

pub const PILL_HEIGHT: f32 = 50.;
const PILL_MAX_WIDTH: f32 = 620.;
const PILL_MIN_WIDTH: f32 = 200.;
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
    /// The highlighted suggestion row. Each change to the typed text starts it over on
    /// "Search for".
    highlight: usize,
    refine_expanded: refine::Expanded,
    refine_choices: Derived<Rc<[refine::Choice]>>,
    suggestion_rows: Derived<Rc<[suggest::Row]>>,
    /// Whether the input shows its placeholder, as last set; `None` to set it again.
    placeholder: Option<bool>,
}

impl PillState {
    /// Drops what was worked out from the library, for new results or a new language.
    pub(super) fn forget_derived(&mut self) {
        self.refine_choices.forget();
        self.suggestion_rows.forget();
    }
}

impl MainView {
    pub(super) fn render_search(
        &mut self,
        show_detail: bool,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let (left, right) = super::content::content_span(window, show_detail);
        let (offset, width) = pill_frame(right - left);
        let pill_left = left + offset;
        let pill = div()
            .relative()
            .child(self.render_pill(pill_left, width, window, cx))
            .children(self.render_suggestions(width, window, cx));
        div()
            .absolute()
            .top(MARGIN)
            .left(px(left))
            .w(px(right - left))
            .flex()
            .flex_col()
            .items_center()
            .gap_2()
            .child(div().w_full().pl(px(offset)).child(pill))
            .child(self.render_refine(right - left - 40., window, cx))
    }

    /// After a language switch, what inputs keep rather than draw: their placeholders.
    /// Tokens draw their labels each frame.
    pub(super) fn translate(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        self.pill.placeholder = None;
        self.pill.forget_derived();
        self.sync_input(window, cx);
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
        if self.library.read(cx).search().typed != search.typed {
            self.pill.highlight = 0;
        }
        self.pill.selected = None;
        self.library
            .update(cx, |library, cx| library.set_search(search, cx));
        self.sync_input(window, cx);
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
            // Only one thing drops down at a time.
            InputEvent::Focus if matches!(self.overlay, Some(Overlay::Display)) => {
                self.set_overlay(None, cx);
            }
            // The pill draws its focus ring and opens or closes the suggestions.
            InputEvent::Focus | InputEvent::Blur => cx.notify(),
            // The pill's keys take ↵ before the input does.
            InputEvent::PressEnter { .. } => {}
        }
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

    /// The input shows the search's typed text, and the placeholder only without tokens.
    /// Also for changes the library makes itself, as switching workspaces does.
    pub(super) fn sync_input(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        let search = self.library.read(cx).search();
        let (typed, placeholder) = (search.typed.clone(), search.tokens.is_empty());
        if self.search.read(cx).value() != typed.as_str() {
            self.search
                .update(cx, |input, cx| input.set_value(typed, window, cx));
        }
        if self.pill.placeholder != Some(placeholder) {
            self.pill.placeholder = Some(placeholder);
            let text = if placeholder {
                t!("main-search-placeholder")
            } else {
                String::new()
            };
            self.search
                .update(cx, |input, cx| input.set_placeholder(text, window, cx));
        }
    }
}

/// Where the pill goes in a content span `span` wide: its offset from the span's left
/// and its width. Centred, with the capsule's room kept free on both sides; when that
/// leaves too little, it gives up the room on the left first, then shrinks, but never
/// reaches into the capsule's.
fn pill_frame(span: f32) -> (f32, f32) {
    let centred = PILL_MAX_WIDTH.min(span - 2. * CAPSULE_ROOM);
    if centred >= PILL_MIN_WIDTH {
        return ((span - centred) / 2., centred);
    }
    let width = PILL_MIN_WIDTH.min(span - CAPSULE_ROOM).max(0.);
    ((span - CAPSULE_ROOM - width).max(0.), width)
}
