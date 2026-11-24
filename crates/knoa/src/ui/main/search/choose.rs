//! Choosing a suggestion: moving the highlight with ↑ ↓, applying it with ↵ or a click,
//! and ⇥ for the first filter row.

use gpui::{Context, Window};
use knoa_browser::Activation;
use knoa_core::Place;

use super::{
    MainView,
    suggest::{Action, Row},
};
use crate::state::search::Token;

impl MainView {
    /// ↑ and ↓ move the highlight while the list is open. Returns whether it was.
    pub(super) fn move_highlight(
        &mut self,
        down: bool,
        window: &Window,
        cx: &mut Context<'_, MainView>,
    ) -> bool {
        let count = self.suggestion_rows(window, cx).len();
        if count == 0 {
            return false;
        }
        let at = self.pill.highlight.min(count - 1);
        self.pill.highlight = if down {
            (at + 1) % count
        } else {
            (at + count - 1) % count
        };
        cx.notify();
        true
    }

    /// ↵: the highlighted row. Returns whether the list was open.
    pub(super) fn apply_highlighted(
        &mut self,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> bool {
        if self.suggestion_rows(window, cx).is_empty() {
            return false;
        }
        self.apply_suggestion(self.pill.highlight, window, cx);
        true
    }

    /// ⇥: the first kind, folder, tag, site or field row. Returns whether there was one.
    pub(super) fn apply_first_filter(
        &mut self,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> bool {
        let rows = self.suggestion_rows(window, cx);
        let Some(index) = rows.iter().position(Row::is_filter) else {
            return false;
        };
        self.apply_suggestion(index, window, cx);
        true
    }

    pub(super) fn apply_suggestion(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        let rows = self.suggestion_rows(window, cx);
        let Some(row) = rows.get(index) else {
            return;
        };
        let mut search = self.library.read(cx).search().clone();
        match row.action.clone() {
            Action::SearchFor => search.lock_in_typed(),
            Action::Add(filter) => {
                search.typed.clear();
                search.add(Token::Filter(filter));
            }
            Action::Folder(id) => {
                search.typed.clear();
                self.show_place(Place::Folder(id), cx);
            }
            Action::Open(id) => {
                self.open_bookmark(id, Activation::Foreground, cx);
                return;
            }
        }
        self.set_search(search, window, cx);
    }
}
