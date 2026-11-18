//! Moving through the events with the keys, in the order the shown view lists them.

use gpui::Context;
use knoa_core::BookmarkId;

use super::EventsPage;
use crate::{
    state::settings::{EventsView, SettingsFile},
    ui::main::MainView,
};

impl EventsPage {
    /// The events in the order the current view lists them.
    fn order(&self, view: EventsView) -> Vec<BookmarkId> {
        let Some(model) = &self.model else {
            return Vec::new();
        };
        let lines = match view {
            EventsView::Calendar => &self.upcoming,
            EventsView::List => &self.list,
        };
        lines
            .order()
            .into_iter()
            .map(|event| model.events[event].id)
            .collect()
    }
}

impl MainView {
    /// The events in the order the page lists them, while it is shown.
    pub(in crate::ui::main) fn events_order(&self, cx: &gpui::App) -> Option<Vec<BookmarkId>> {
        let showing = self.library.read(cx).place() == knoa_core::Place::Events;
        showing.then(|| self.events.order(SettingsFile::get(cx).events_view))
    }

    pub(super) fn step_event(&mut self, by: isize, cx: &mut Context<'_, MainView>) {
        let Some(order) = self.events_order(cx) else {
            return;
        };
        let at = self
            .selected
            .and_then(|id| order.iter().position(|e| *e == id));
        let next = match at {
            None => 0,
            Some(at) => at
                .saturating_add_signed(by)
                .min(order.len().saturating_sub(1)),
        };
        let Some(&id) = order.get(next) else {
            return;
        };
        self.set_selected(Some(id), cx);
        self.reveal_event(id, cx);
        cx.notify();
    }

    fn reveal_event(&self, id: BookmarkId, cx: &Context<'_, MainView>) {
        let page = &self.events;
        let Some(model) = &page.model else {
            return;
        };
        let Some(event) = model.events.iter().position(|e| e.id == id) else {
            return;
        };
        match SettingsFile::get(cx).events_view {
            EventsView::Calendar => {
                let origin = page.placement.origin();
                page.upcoming.reveal(event, origin, &page.upcoming_scroll);
            }
            EventsView::List => page.list.reveal(event, 0., &self.scroll),
        }
    }
}
