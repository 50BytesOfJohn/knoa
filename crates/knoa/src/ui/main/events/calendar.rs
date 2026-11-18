//! The month grid: six weeks from the region's first weekday under their weekday names,
//! with the Upcoming panel beside it, or under it where the page is too narrow.

use gpui::{
    AnyElement, Context, Div, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div, px,
};
use jiff::{
    ToSpan as _,
    civil::{Date, Weekday},
};

use super::{
    labels,
    upcoming::{self, Placement},
};
use crate::ui::{
    main::MainView,
    theme::{CARD_RADIUS, GUTTER, Theme},
};

const WEEKS: usize = 6;
const WEEKDAY_ROW: f32 = 26.;
/// The grid's border, top and bottom.
const BORDERS: f32 = 2.;
/// Narrower than this beside the panel, the grid takes the whole width and the panel goes
/// under it.
const MIN_GRID_WIDTH: f32 = 480.;
/// How much of the panel shows under a stacked grid before scrolling, so it is found.
const PANEL_PEEK: f32 = 96.;
/// The shortest a stacked grid gets, so its days keep room for a chip or two.
const MIN_STACKED_GRID: f32 = 420.;

impl MainView {
    pub(super) fn calendar_body(
        &mut self,
        width: f32,
        height: f32,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        let gutter = f32::from(GUTTER);
        let beside = width - upcoming::WIDTH - gutter;
        if beside >= MIN_GRID_WIDTH {
            self.events.placement = Placement::Beside;
            return div()
                .size_full()
                .flex()
                .gap(GUTTER)
                .child(self.month_grid(beside, height, cx))
                .child(self.upcoming_panel(Placement::Beside, height, cx))
                .into_any_element();
        }
        let grid_height = (height - PANEL_PEEK).max(MIN_STACKED_GRID);
        let placement = Placement::Below {
            origin: grid_height + gutter + upcoming::BORDER + upcoming::TITLE_HEIGHT,
        };
        self.events.placement = placement;
        div()
            .id("events-stack")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.events.upcoming_scroll)
            .on_scroll(cx.listener(|_, _, _, cx| cx.notify()))
            .flex()
            .flex_col()
            .gap(GUTTER)
            .child(self.month_grid(width, grid_height, cx))
            .child(self.upcoming_panel(placement, height, cx))
            .into_any_element()
    }

    fn month_grid(&self, width: f32, height: f32, cx: &mut Context<'_, MainView>) -> AnyElement {
        let theme = Theme::get(cx).clone();
        let page = &self.events;
        let Some(model) = &page.model else {
            return div().into_any_element();
        };
        let cell_height = (height - WEEKDAY_ROW - BORDERS) / WEEKS as f32;
        let days: Vec<Date> = grid_days(page.month, page.week_start).collect();
        let mut grid = div().flex_1().flex().flex_col();
        for week in days.chunks(7) {
            // On the row, not the cells: a dimmed day outside the month would dim its line.
            let mut row = div()
                .flex()
                .h(px(cell_height))
                .border_t_1()
                .border_color(theme.hairline);
            for &day in week {
                row = row.child(self.day_cell(model, day, cell_height, cx));
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
            .child(weekday_row(page.week_start, &theme))
            .child(grid)
            .into_any_element()
    }
}

fn weekday_row(week_start: Weekday, theme: &Theme) -> Div {
    let mut header = div().flex().h(px(WEEKDAY_ROW));
    for day in week_start.cycle_forward().take(7) {
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
    header
}

/// The 42 days the grid shows for the month starting `first`.
fn grid_days(first: Date, week_start: Weekday) -> impl Iterator<Item = Date> {
    let lead = first.weekday().since(week_start);
    let start = first.checked_sub(i64::from(lead).days()).unwrap_or(first);
    start.series(1.day()).take(WEEKS * 7)
}
