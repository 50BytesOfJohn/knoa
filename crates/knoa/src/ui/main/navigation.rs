//! Moving the selection through the masonry grid with the arrow keys.

use gpui::{Context, Window, point, px};
use knoa_core::BookmarkId;

use super::{
    MainView, MoveDown, MoveLeft, MoveRight, MoveUp, masonry::Placed, search::SEARCH_AREA_HEIGHT,
};

#[derive(Clone, Copy)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl MainView {
    pub(super) fn move_left(
        &mut self,
        _: &MoveLeft,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        self.step(Direction::Left, window, cx);
    }

    pub(super) fn move_right(
        &mut self,
        _: &MoveRight,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        self.step(Direction::Right, window, cx);
    }

    pub(super) fn move_up(
        &mut self,
        _: &MoveUp,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        self.step(Direction::Up, window, cx);
    }

    pub(super) fn move_down(
        &mut self,
        _: &MoveDown,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        self.step(Direction::Down, window, cx);
    }

    pub(super) fn select_first(&mut self, cx: &mut Context<'_, MainView>) {
        if let Some(first) = self.layout.results.first() {
            self.selected = Some(first.id);
            cx.notify();
        }
    }

    fn step(&mut self, direction: Direction, window: &mut Window, cx: &mut Context<'_, MainView>) {
        let cards = &self.layout.layout.cards;
        let current = self.selected.and_then(|id| {
            cards
                .iter()
                .position(|c| self.layout.results.get(c.index).is_some_and(|b| b.id == id))
        });
        let next = match current {
            None => (!cards.is_empty()).then_some(0),
            Some(at) => neighbour(cards, at, direction),
        };
        let Some(next) = next else { return };
        let placed = cards[next];
        if let Some(bookmark) = self.layout.results.get(placed.index) {
            self.selected = Some(bookmark.id);
        }
        self.reveal(&placed, window);
        cx.notify();
    }

    /// The card after `id` in reading order, or the one before it at the end.
    pub(super) fn neighbour_of(&self, id: BookmarkId) -> Option<BookmarkId> {
        let cards = &self.layout.layout.cards;
        let results = &self.layout.results;
        let at = cards
            .iter()
            .position(|c| results.get(c.index).is_some_and(|b| b.id == id))?;
        let next = if at + 1 < cards.len() {
            at + 1
        } else {
            at.checked_sub(1)?
        };
        results.get(cards[next].index).map(|b| b.id)
    }

    /// Scrolls just enough to bring the card fully into view.
    fn reveal(&self, card: &Placed, window: &Window) {
        let top_inset = SEARCH_AREA_HEIGHT + 12.;
        let viewport = f32::from(window.viewport_size().height);
        let scroll_top = -f32::from(self.scroll.offset().y);
        let card_top = card.y + top_inset;
        let card_bottom = card_top + card.height;
        let new_top = if card_top - top_inset < scroll_top {
            card_top - top_inset - 12.
        } else if card_bottom > scroll_top + viewport {
            card_bottom - viewport + 24.
        } else {
            return;
        };
        self.scroll.set_offset(point(px(0.), px(-new_top.max(0.))));
    }
}

/// Left and right follow reading order; up and down stay in the column.
fn neighbour(cards: &[Placed], at: usize, direction: Direction) -> Option<usize> {
    let card = cards[at];
    match direction {
        Direction::Left => at.checked_sub(1),
        Direction::Right => (at + 1 < cards.len()).then_some(at + 1),
        Direction::Up => cards
            .iter()
            .enumerate()
            .filter(|(_, c)| (c.x - card.x).abs() < 1. && c.y < card.y)
            .max_by(|a, b| a.1.y.total_cmp(&b.1.y))
            .map(|(i, _)| i),
        Direction::Down => cards
            .iter()
            .enumerate()
            .filter(|(_, c)| (c.x - card.x).abs() < 1. && c.y > card.y)
            .min_by(|a, b| a.1.y.total_cmp(&b.1.y))
            .map(|(i, _)| i),
    }
}
