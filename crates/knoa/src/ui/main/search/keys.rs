//! The pill's keys: ⌫ on an empty input, Esc, the suggestion list's ↑ ↓ ↵ ⇥, and ⌘C
//! without a selection. The input's cursor moves pass through, clearing a selected token.
//!
//! GPUI runs a key's binding before any key-down listener, and the input binds every one
//! of these keys. So they are caught as the input's own actions, in the capture phase on
//! the way down to it: a handler that acts stops the action there, one that doesn't lets
//! the input have it.

use gpui::{
    Action, ClipboardItem, Context, Div, InteractiveElement as _, KeyDownEvent, Stateful, Window,
};
use gpui_component::input::{
    Backspace, Copy, Enter, Escape, IndentInline, MoveDown, MoveEnd, MoveHome, MoveLeft, MoveRight,
    MoveToEnd, MoveToNextWord, MoveToPreviousWord, MoveToStart, MoveUp,
};
use knoa_core::Place;

use super::{MainView, Slot};
use crate::i18n;

impl MainView {
    pub(super) fn with_pill_keys(
        &self,
        pill: Stateful<Div>,
        cx: &mut Context<'_, MainView>,
    ) -> Stateful<Div> {
        let pill = self.deselect_on_moves(pill, cx);
        pill.capture_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
            // Typing goes on past a selected token rather than removing it.
            if event.keystroke.key != "backspace" {
                this.deselect_token(cx);
            }
        }))
        .capture_action(cx.listener(|this, _: &Backspace, window, cx| {
            if this.search.read(cx).value().is_empty() && this.backspace_token(window, cx) {
                cx.stop_propagation();
            }
        }))
        .capture_action(cx.listener(|this, _: &Escape, window, cx| {
            this.deselect_token(cx);
            if !this.collapse_refine(cx) {
                this.escape_pill(window, cx);
            }
            cx.stop_propagation();
        }))
        .capture_action(cx.listener(|this, _: &MoveUp, window, cx| {
            this.deselect_token(cx);
            if this.move_highlight(false, window, cx) {
                cx.stop_propagation();
            }
        }))
        .capture_action(cx.listener(|this, _: &MoveDown, window, cx| {
            this.deselect_token(cx);
            if !this.move_highlight(true, window, cx) {
                this.select_first(cx);
                this.cards_focus.focus(window, cx);
            }
            cx.stop_propagation();
        }))
        .capture_action(cx.listener(|this, enter: &Enter, window, cx| {
            this.deselect_token(cx);
            if !enter.secondary && !enter.shift && this.apply_highlighted(window, cx) {
                cx.stop_propagation();
            }
        }))
        // ⇥ with nothing to apply keeps its usual meaning.
        .capture_action(cx.listener(|this, _: &IndentInline, window, cx| {
            this.deselect_token(cx);
            if this.apply_first_filter(window, cx) {
                cx.stop_propagation();
            }
        }))
        .capture_action(cx.listener(|this, _: &Copy, _, cx| {
            if !this.input_has_selection(cx) {
                this.copy_search(cx);
                cx.stop_propagation();
            }
        }))
    }

    fn deselect_on_moves(
        &self,
        pill: Stateful<Div>,
        cx: &mut Context<'_, MainView>,
    ) -> Stateful<Div> {
        let pill = Self::deselect_on::<MoveLeft>(pill, cx);
        let pill = Self::deselect_on::<MoveRight>(pill, cx);
        let pill = Self::deselect_on::<MoveHome>(pill, cx);
        let pill = Self::deselect_on::<MoveEnd>(pill, cx);
        let pill = Self::deselect_on::<MoveToStart>(pill, cx);
        let pill = Self::deselect_on::<MoveToEnd>(pill, cx);
        let pill = Self::deselect_on::<MoveToPreviousWord>(pill, cx);
        Self::deselect_on::<MoveToNextWord>(pill, cx)
    }

    /// Clears a selected token on the input's action `A` and lets the input have it.
    fn deselect_on<A: Action>(
        pill: Stateful<Div>,
        cx: &mut Context<'_, MainView>,
    ) -> Stateful<Div> {
        pill.capture_action(cx.listener(|this, _: &A, _, cx| this.deselect_token(cx)))
    }

    fn deselect_token(&mut self, cx: &mut Context<'_, MainView>) {
        if self.pill.selected.take().is_some() {
            cx.notify();
        }
    }

    /// The first ⌫ selects the last token, the second removes it. Returns whether there
    /// was a token to act on.
    fn backspace_token(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) -> bool {
        let library = self.library.read(cx);
        let last = match library.search().tokens.len() {
            0 if library.place() == Place::Everything => return false,
            0 => Slot::Scope,
            n => Slot::Token(n - 1),
        };
        if self.pill.selected == Some(last) {
            self.remove_token(last, window, cx);
            self.pill.selected = None;
        } else {
            self.pill.selected = Some(last);
        }
        cx.notify();
        true
    }

    /// Clears the typed text, then the tokens other than the scope, then goes to the
    /// cards.
    fn escape_pill(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        let search = self.library.read(cx).search();
        if self.overlay.is_some() {
            self.set_overlay(None, cx);
        } else if !search.typed.is_empty() {
            let mut search = search.clone();
            search.typed.clear();
            self.set_search(search, window, cx);
        } else if !search.tokens.is_empty() {
            self.clear_search(window, cx);
        } else {
            self.cards_focus.focus(window, cx);
        }
    }

    fn input_has_selection(&self, cx: &Context<'_, MainView>) -> bool {
        !self.search.read(cx).selected_range().is_empty()
    }

    /// The whole search as it would be typed, with kinds as slugs, the typed form.
    fn copy_search(&self, cx: &mut Context<'_, MainView>) {
        let typed = self.library.read(cx).search().typed_form();
        cx.write_to_clipboard(ClipboardItem::new_string(i18n::with_kind_slugs(&typed)));
    }
}
