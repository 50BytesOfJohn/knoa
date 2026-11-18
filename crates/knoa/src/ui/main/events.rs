//! The Events place: a month calendar with the upcoming events beside it, or one list of
//! them, as the `events_view` setting on this Mac says.

mod calendar;
mod day_cell;
mod dots;
mod header;
mod labels;
mod lines;
mod list;
mod list_row;
mod model;
mod rows;
mod selection;
mod span;
mod upcoming;
mod week_start;

use std::sync::Arc;

use gpui::{
    AnyElement, Context, Div, InteractiveElement as _, IntoElement, ParentElement as _,
    ScrollHandle, Stateful, Styled as _, Window, div, px,
};
use jiff::{
    ToSpan as _, Zoned,
    civil::{Date, Weekday},
};

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
pub(super) use span::local_time;
use upcoming::Placement;

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
    /// Scrolls the Upcoming panel, and the grid too while the panel is under it.
    upcoming_scroll: ScrollHandle,
    /// Where the Upcoming panel was drawn last.
    placement: Placement,
}

/// The model is rebuilt for new results, and each minute so events move on to Happening
/// now and Past, and a new day moves Today, without a restart.
#[derive(PartialEq)]
struct BuildKey {
    results: usize,
    minute: i64,
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
            placement: Placement::Beside,
        }
    }

    fn update(&mut self, results: Arc<Vec<knoa_core::Bookmark>>) {
        let now = Zoned::now();
        let key = BuildKey {
            results: Arc::as_ptr(&results) as usize,
            minute: now.timestamp().as_second().div_euclid(60),
        };
        if self.built_for.as_ref() == Some(&key) {
            return;
        }
        let model = Model::build(results, &now);
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
        // Back to the top of the panel, where the day's events now are, unless the grid
        // above it is in view.
        let top = -f32::from(self.upcoming_scroll.offset().y);
        let top = top.min(self.placement.origin());
        self.upcoming_scroll
            .set_offset(gpui::point(px(0.), px(-top)));
    }

    fn move_month(&mut self, months: i32) {
        if let Ok(month) = self.month.checked_add(months.months()) {
            self.month = month.first_of_month();
        }
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
        let body_height = f32::from(window.viewport_size().height)
            - cards_top()
            - header::HEIGHT
            - f32::from(GUTTER);
        let body = self.events_body(view, right - left, body_height, window, cx);
        let page = with_actions(div().id("events"), cx)
            .key_context("Cards Events")
            .track_focus(&self.cards_focus)
            .size_full()
            .pt(px(cards_top()))
            .flex()
            .flex_col()
            .child(self.events_header(view, right - left, cx))
            .child(div().h(px(body_height)).child(body));
        div()
            .absolute()
            .top_0()
            .bottom_0()
            .left(px(left))
            .w(px(right - left))
            .child(page)
            .into_any_element()
    }

    fn events_body(
        &mut self,
        view: EventsView,
        width: f32,
        height: f32,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        let empty = self
            .events
            .model
            .as_ref()
            .is_none_or(|model| model.events.is_empty());
        if empty {
            self.keep_no_images(window, cx);
            return self.empty_state(cx).into_any_element();
        }
        match view {
            EventsView::Calendar => {
                // Chips and the panel's rows draw no pictures.
                self.keep_no_images(window, cx);
                self.calendar_body(width, height, cx)
            }
            EventsView::List => self.events_list(height, window, cx),
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

    /// Picks `day`, or lets go of it when it was picked.
    fn toggle_event_day(&mut self, day: Date, cx: &mut Context<'_, Self>) {
        let day = (self.events.day != Some(day)).then_some(day);
        self.events.select_day(day);
        cx.notify();
    }
}

/// The page's keys: months, moving through events, and the actions on the selected one.
fn with_actions(page: Stateful<Div>, cx: &mut Context<'_, MainView>) -> Stateful<Div> {
    page.on_action(cx.listener(MainView::previous_month))
        .on_action(cx.listener(MainView::next_month))
        .on_action(cx.listener(MainView::show_today))
        .on_action(cx.listener(|this, _: &super::MoveUp, _, cx| {
            this.step_event(-1, cx);
        }))
        .on_action(cx.listener(|this, _: &super::MoveDown, _, cx| {
            this.step_event(1, cx);
        }))
        .on_action(cx.listener(MainView::open_selected))
        .on_action(cx.listener(MainView::open_selected_in_background))
        .on_action(cx.listener(MainView::quick_look))
        .on_action(cx.listener(MainView::toggle_up_next))
        .on_action(cx.listener(MainView::copy_link))
        .on_action(cx.listener(MainView::delete_selected))
        .on_action(cx.listener(MainView::dismiss))
}
