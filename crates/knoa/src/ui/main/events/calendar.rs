//! The month grid: six weeks from the region's first weekday, each day with its first few
//! events as chips, and the Upcoming panel beside it while there is room.

use gpui::{
    AnyElement, ClickEvent, Context, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, StatefulInteractiveElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};
use jiff::{
    ToSpan as _,
    civil::{Date, Weekday},
};

use super::{labels, model::Model, upcoming};
use crate::ui::{
    main::{MainView, bookmark_menu::with_bookmark_menu},
    theme::{CARD_RADIUS, GUTTER, Theme},
};

const WEEKS: usize = 6;
const WEEKDAY_ROW: f32 = 26.;
const DAY_NUMBER: f32 = 26.;
const CHIP: f32 = 18.;
const CHIP_GAP: f32 = 2.;
/// The most chips a day shows before "+N more".
const MAX_CHIPS: usize = 3;
/// Narrower than this, the grid takes the whole width and the Upcoming panel steps aside.
const MIN_GRID_WIDTH: f32 = 480.;

/// The 42 days the grid shows for the month starting `first`.
fn grid_days(first: Date, week_start: Weekday) -> impl Iterator<Item = Date> {
    let lead = first.weekday().since(week_start);
    let start = first.checked_sub(i64::from(lead).days()).unwrap_or(first);
    start.series(1.day()).take(WEEKS * 7)
}

impl MainView {
    pub(super) fn calendar_body(
        &mut self,
        width: f32,
        height: f32,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        // Chips and the panel's rows draw no pictures.
        self.keep_no_images(window, cx);
        let side = width - upcoming::WIDTH - f32::from(GUTTER) >= MIN_GRID_WIDTH;
        let grid_width = if side {
            width - upcoming::WIDTH - f32::from(GUTTER)
        } else {
            width
        };
        div()
            .size_full()
            .flex()
            .gap(GUTTER)
            .child(self.month_grid(grid_width, height, cx))
            .when(side, |row| row.child(self.upcoming_panel(height, cx)))
            .into_any_element()
    }

    fn month_grid(&self, width: f32, height: f32, cx: &mut Context<'_, MainView>) -> AnyElement {
        let theme = Theme::get(cx).clone();
        let page = &self.events;
        let Some(model) = &page.model else {
            return div().into_any_element();
        };
        // Less the grid's 1 px border, top and bottom.
        let cell_height = (height - WEEKDAY_ROW - 2.) / WEEKS as f32;
        let mut header = div().flex().h(px(WEEKDAY_ROW));
        for day in page.week_start.cycle_forward().take(7) {
            header = header.child(
                div()
                    .flex_1()
                    .px_2()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.muted)
                    .child(labels::short_weekday(day)),
            );
        }
        let days: Vec<Date> = grid_days(page.month, page.week_start).collect();
        let mut grid = div().flex_1().flex().flex_col();
        for week in days.chunks(7) {
            let mut row = div().flex().h(px(cell_height));
            for &day in week {
                row = row.child(self.day_cell(model, day, cell_height, &theme, cx));
            }
            grid = grid.child(row);
        }
        div()
            .flex_none()
            .w(px(width))
            .h(px(height))
            .flex()
            .flex_col()
            .rounded(CARD_RADIUS)
            .bg(theme.card)
            .border_1()
            .border_color(theme.hairline)
            .overflow_hidden()
            .child(header)
            .child(grid)
            .into_any_element()
    }

    fn day_cell(
        &self,
        model: &Model,
        day: Date,
        height: f32,
        theme: &Theme,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        let page = &self.events;
        let in_month = day.year() == page.month.year() && day.month() == page.month.month();
        let today = day == model.today();
        let picked = page.day == Some(day);
        let events = model.on(day);
        let room = ((height - DAY_NUMBER - 4.) / (CHIP + CHIP_GAP))
            .floor()
            .max(0.) as usize;
        let shown = if events.len() <= room.min(MAX_CHIPS) {
            events.len()
        } else {
            room.min(MAX_CHIPS + 1).saturating_sub(1)
        };
        let mut cell = div()
            .id(("day", day_id(day)))
            .flex_1()
            .min_w_0()
            .h(px(height))
            .px_1()
            .pb_1()
            .flex()
            .flex_col()
            .gap(px(CHIP_GAP))
            .border_t_1()
            .border_color(theme.hairline)
            .overflow_hidden()
            .cursor_pointer()
            .when(picked, |s| s.bg(theme.selected_row()))
            .when(!in_month, |s| s.opacity(0.45))
            .child(day_number(day, today, theme))
            .on_click(cx.listener(move |this, _, _, cx| this.select_event_day(day, cx)));
        for &event in &events[..shown] {
            cell = cell.child(self.chip(model, event, day, theme, cx));
        }
        if shown < events.len() {
            cell = cell.child(
                div()
                    .h(px(CHIP))
                    .px_1p5()
                    .text_xs()
                    .text_color(theme.muted)
                    .child(format!("+{} more", events.len() - shown)),
            );
        }
        cell.into_any_element()
    }

    /// One event on one day: its start time on the day it starts, then its title.
    fn chip(
        &self,
        model: &Model,
        event: usize,
        day: Date,
        theme: &Theme,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        let bookmark = model.bookmark(event);
        let id = bookmark.id;
        let span = model.events[event].span;
        let time = span
            .filter(|s| !s.all_day && s.start.date() == day)
            .map(|s| labels::start_time(&s));
        let past = model.is_past(event);
        let selected = self.selected == Some(id);
        let (bg, fg) = if selected {
            (theme.accent, theme.on_accent)
        } else {
            (theme.accent.opacity(0.12), theme.text)
        };
        let hover = theme.accent.opacity(0.2);
        let chip = div()
            .id(("chip", event))
            .h(px(CHIP))
            .px_1p5()
            .flex()
            .items_center()
            .gap_1()
            .rounded(px(4.))
            .bg(bg)
            .text_color(fg)
            .text_xs()
            .when(!selected, |s| s.hover(move |s| s.bg(hover)))
            .when(past, |s| s.opacity(0.55))
            .when_some(time, |s, time| {
                s.child(
                    div()
                        .flex_none()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(time),
                )
            })
            .child(div().min_w_0().truncate().child(bookmark.title.clone()))
            .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                cx.stop_propagation();
                this.click_card(id, event, window, cx);
            }));
        with_bookmark_menu(chip, id, cx).into_any_element()
    }
}

fn day_number(day: Date, today: bool, theme: &Theme) -> impl IntoElement {
    let number = div()
        .size(px(22.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .text_xs()
        .child(day.day().to_string());
    let number = if today {
        number
            .bg(theme.accent)
            .text_color(theme.on_accent)
            .font_weight(FontWeight::SEMIBOLD)
    } else {
        number.text_color(theme.text)
    };
    div().h(px(DAY_NUMBER)).flex().items_center().child(number)
}

/// A day as a number for element ids, unique within the grid.
fn day_id(day: Date) -> usize {
    day.year() as usize * 400 + day.month() as usize * 32 + day.day() as usize
}
