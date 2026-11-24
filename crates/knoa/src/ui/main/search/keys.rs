//! The pill's keys, caught before the input sees them: ⌫ on an empty input, Esc, ↓, ⇥
//! and ⌘C without a selection.

use gpui::{ClipboardItem, Context, KeyDownEvent, Window};
use knoa_core::Place;

use super::{MainView, Slot};
use crate::i18n;

impl MainView {
    pub(super) fn pill_keys(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        let keys = &event.keystroke;
        let plain = !keys.modifiers.modified();
        let key = keys.key.as_str();
        if key != "backspace" && self.pill.selected.take().is_some() {
            cx.notify();
        }
        let handled = match key {
            "backspace" if plain && self.search.read(cx).value().is_empty() => {
                self.backspace_token(window, cx)
            }
            "escape" if plain => {
                self.escape_pill(window, cx);
                true
            }
            "down" if plain => {
                self.select_first(cx);
                self.cards_focus.focus(window, cx);
                true
            }
            // Focus stays in the pill.
            "tab" if plain => true,
            "c" if keys.modifiers.platform && !self.input_has_selection(cx) => {
                self.copy_search(cx);
                true
            }
            _ => false,
        };
        if handled {
            cx.stop_propagation();
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
