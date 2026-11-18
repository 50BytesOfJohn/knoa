//! The Events place: a month calendar with the upcoming events beside it, or one list of
//! them, as the `events_view` setting on this Mac says.

mod calendar;
mod header;
mod labels;
mod lines;
mod list;
mod model;
mod upcoming;
mod week_start;

use std::sync::Arc;

use gpui::{
    AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _, ScrollHandle,
    Styled as _, Window, div, px,
};
use jiff::{
    ToSpan as _, Zoned,
    civil::{Date, DateTime, Weekday},
};
use knoa_core::BookmarkId;

use super::{
    MainView,
    content::{LayoutCache, cards_top, content_span},
    keys::{NextMonth, PreviousMonth, ShowToday},
};
use crate::{
    state::settings::{EventsView, SettingsFile},
    ui::theme::GUTTER,
};
pub(super) use header::view_switch;
use lines::Lines;
use model::Model;
pub(super) use model::local_time;

/// The Events page's own state: what it built from the results, and where the user is.
pub struct EventsPage {
    built_for: Option<BuildKey>,
    model: Option<Model>,
    /// The first day of the month the calendar shows.
    month: Date,
    /// The day clicked in the calendar, whose events lead the upcoming list.
    day: Option<Date>,
    week_start: Weekday,
    list: Lines,
    upcoming: Lines,
    upcoming_scroll: ScrollHandle,
}

/// The model is rebuilt for new results, and each minute so events move on to Happening
/// now and Past, and a new day moves Today, without a restart.
#[derive(PartialEq)]
struct BuildKey {
    results: usize,
    minute: DateTime,
}

impl EventsPage {
    pub fn new() -> EventsPage {
        EventsPage {
            built_for: None,
            model: None,
            month: Zoned::now().date().first_of_month(),
            day: None,
            week_start: week_start::first_weekday(),
            list: Lines::default(),
            upcoming: Lines::default(),
            upcoming_scroll: ScrollHandle::new(),
        }
    }

    fn update(&mut self, results: Arc<Vec<knoa_core::Bookmark>>) {
        let now = Zoned::now();
        let minute = now
            .datetime()
            .with()
            .second(0)
            .subsec_nanosecond(0)
            .build()
            .unwrap_or(now.datetime());
        let key = BuildKey {
            results: Arc::as_ptr(&results) as usize,
            minute,
        };
        if self.built_for.as_ref() == Some(&key) {
            return;
        }
        let model = Model::build(results, minute, now.time_zone());
        self.week_start = week_start::first_weekday();
        self.list = list::lines(&model);
        self.upcoming = upcoming::lines(&model, self.day);
        self.model = Some(model);
        self.built_for = Some(key);
    }

    fn select_day(&mut self, day: Option<Date>) {
        self.day = day;
        if let Some(model) = &self.model {
            self.upcoming = upcoming::lines(model, day);
        }
        self.upcoming_scroll.set_offset(gpui::point(px(0.), px(0.)));
    }

    fn move_month(&mut self, months: i32) {
        if let Ok(month) = self.month.checked_add(months.months()) {
            self.month = month.first_of_month();
        }
    }

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

/// Saves how Events shows, and redraws.
pub fn set_events_view(view: EventsView, cx: &mut gpui::App) {
    SettingsFile::update(cx, |s| s.events_view = view);
    cx.refresh_windows();
}

impl MainView {
    pub(super) fn render_events(
        &mut self,
        show_detail: bool,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        // The cards' layout belongs to another place: drop it so nothing reads it here.
        self.layout = LayoutCache::default();
        let results = self.library.read(cx).results();
        self.events.update(results);
        let view = SettingsFile::get(cx).events_view;
        let (left, right) = content_span(window, show_detail);
        let width = right - left;
        let body_height = f32::from(window.viewport_size().height)
            - cards_top()
            - header::HEIGHT
            - f32::from(GUTTER);
        let empty = self
            .events
            .model
            .as_ref()
            .is_none_or(|model| model.events.is_empty());
        let body = if empty {
            self.keep_no_images(window, cx);
            self.empty_state(cx).into_any_element()
        } else {
            match view {
                EventsView::Calendar => self.calendar_body(width, body_height, window, cx),
                EventsView::List => self.events_list(body_height, window, cx),
            }
        };
        div()
            .absolute()
            .top_0()
            .bottom_0()
            .left(px(left))
            .w(px(width))
            .child(
                div()
                    .id("events")
                    .key_context("Cards Events")
                    .track_focus(&self.cards_focus)
                    .size_full()
                    .pt(px(cards_top()))
                    .flex()
                    .flex_col()
                    .on_action(cx.listener(Self::previous_month))
                    .on_action(cx.listener(Self::next_month))
                    .on_action(cx.listener(Self::show_today))
                    .on_action(cx.listener(|this, _: &super::MoveUp, _, cx| {
                        this.step_event(-1, cx);
                    }))
                    .on_action(cx.listener(|this, _: &super::MoveDown, _, cx| {
                        this.step_event(1, cx);
                    }))
                    .on_action(cx.listener(Self::open_selected))
                    .on_action(cx.listener(Self::open_selected_in_background))
                    .on_action(cx.listener(Self::quick_look))
                    .on_action(cx.listener(Self::toggle_up_next))
                    .on_action(cx.listener(Self::copy_link))
                    .on_action(cx.listener(Self::delete_selected))
                    .on_action(cx.listener(Self::dismiss))
                    .child(self.events_header(view, cx))
                    .child(div().h(px(body_height)).child(body)),
            )
            .into_any_element()
    }

    /// The events in the order the page lists them, while it is shown.
    pub(super) fn events_order(&self, cx: &gpui::App) -> Option<Vec<BookmarkId>> {
        let showing = self.library.read(cx).place() == knoa_core::Place::Events;
        showing.then(|| self.events.order(SettingsFile::get(cx).events_view))
    }

    fn step_event(&mut self, by: isize, cx: &mut Context<'_, MainView>) {
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
            EventsView::Calendar => page.upcoming.reveal(event, &page.upcoming_scroll),
            EventsView::List => page.list.reveal(event, &self.scroll),
        }
    }

    fn previous_month(&mut self, _: &PreviousMonth, _: &mut Window, cx: &mut Context<'_, Self>) {
        self.events.move_month(-1);
        cx.notify();
    }

    fn next_month(&mut self, _: &NextMonth, _: &mut Window, cx: &mut Context<'_, Self>) {
        self.events.move_month(1);
        cx.notify();
    }

    fn show_today(&mut self, _: &ShowToday, _: &mut Window, cx: &mut Context<'_, Self>) {
        self.events.month = Zoned::now().date().first_of_month();
        self.events.select_day(None);
        cx.notify();
    }

    fn select_event_day(&mut self, day: Date, cx: &mut Context<'_, Self>) {
        let day = (self.events.day != Some(day)).then_some(day);
        self.events.select_day(day);
        cx.notify();
    }
}
