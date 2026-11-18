//! The Upcoming panel by the calendar: the clicked day's events, then what is on now and
//! what is coming, by day, then the events with no date.

use gpui::{
    AnyElement, Context, Div, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    Pixels, StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};
use jiff::civil::Date;

use super::{
    labels,
    lines::{Heights, Line, Lines},
    model::Model,
    rows::{self, RowStyle},
};
use crate::ui::{
    components::site_mark,
    main::{MainView, bookmark_menu::with_bookmark_menu},
    theme::{PANEL_RADIUS, Theme},
};

pub const WIDTH: f32 = 300.;
pub const TITLE_HEIGHT: f32 = 44.;
pub const BORDER: f32 = 1.;
const HEIGHTS: Heights = Heights {
    month: 0.,
    heading: 30.,
    event: 54.,
    note: 60.,
};
const INSET: Pixels = px(8.);
const STYLE: RowStyle = RowStyle {
    inset: INSET,
    past_opacity: 0.55,
};
/// Rows built above and below the panel's view.
const OVERSCAN: f32 = 300.;
/// Space under the last row.
const FOOT: f32 = 8.;

/// Where the panel sits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Placement {
    /// Beside the grid, as tall as it, scrolling on its own.
    Beside,
    /// Under the grid, as tall as its rows, scrolling with the grid. Its rows start
    /// `origin` down the scrolled content.
    Below { origin: f32 },
}

impl Placement {
    /// How far down the scrolled content the panel's rows start.
    pub fn origin(self) -> f32 {
        match self {
            Placement::Beside => 0.,
            Placement::Below { origin } => origin,
        }
    }
}

/// The panel's lines for `model`, with `day`'s events first when a day is picked.
pub fn lines(model: &Model, day: Option<Date>) -> Lines {
    let today = model.today();
    let mut lines = Lines::default();
    if let Some(day) = day {
        let events = model.on(day);
        section(&mut lines, labels::day_heading(day, today), events);
        if events.is_empty() {
            lines.push(Line::Note("Nothing on this day.".into()), &HEIGHTS);
        }
    }
    if !model.happening.is_empty() {
        section(&mut lines, "Happening now".into(), &model.happening);
    }
    for (start, events) in &model.upcoming {
        section(&mut lines, labels::day_heading(*start, today), events);
    }
    if model.happening.is_empty() && model.upcoming.is_empty() {
        let note = "Nothing coming up. Events you save show here until they end.";
        lines.push(Line::Note(note.into()), &HEIGHTS);
    }
    if !model.undated.is_empty() {
        section(&mut lines, "Date not set".into(), &model.undated);
    }
    lines
}

fn section(lines: &mut Lines, heading: String, events: &[usize]) {
    lines.push(Line::Heading(heading.into()), &HEIGHTS);
    for &event in events {
        lines.push(Line::Event(event), &HEIGHTS);
    }
}

impl MainView {
    /// The panel, `view_height` tall where it scrolls on its own, else as tall as the
    /// window's view of it.
    pub(super) fn upcoming_panel(
        &self,
        placement: Placement,
        view_height: f32,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        let theme = Theme::get(cx).clone();
        let page = &self.events;
        let rows = self.upcoming_rows(placement, view_height, cx);
        let frame = div()
            .flex_none()
            .flex()
            .flex_col()
            .rounded(PANEL_RADIUS)
            .bg(theme.panel)
            .border(px(BORDER))
            .border_color(theme.hairline)
            .overflow_hidden()
            .child(title());
        match placement {
            Placement::Beside => frame.w(px(WIDTH)).h(px(view_height)).child(
                div()
                    .id("upcoming")
                    .flex_1()
                    .overflow_y_scroll()
                    .track_scroll(&page.upcoming_scroll)
                    .on_scroll(cx.listener(|_, _, _, cx| cx.notify()))
                    .child(rows),
            ),
            Placement::Below { .. } => frame.w_full().child(rows),
        }
        .into_any_element()
    }

    /// The rows in view, placed on a canvas as tall as all of them.
    fn upcoming_rows(
        &self,
        placement: Placement,
        view_height: f32,
        cx: &mut Context<'_, MainView>,
    ) -> Div {
        let theme = Theme::get(cx).clone();
        let page = &self.events;
        let mut canvas = div().relative().w_full().h(px(page.upcoming.height));
        let Some(model) = &page.model else {
            return canvas;
        };
        let top = -f32::from(page.upcoming_scroll.offset().y) - placement.origin();
        let band = page
            .upcoming
            .visible(top - OVERSCAN, top + view_height + OVERSCAN);
        for (index, placed) in band {
            let line = match placed.line {
                Line::Event(event) => self.upcoming_row(index, model, event, cx),
                _ => rows::text_line(&placed.line, INSET, &theme),
            };
            canvas = canvas.child(rows::placed(placed, line));
        }
        div().px_2().pb(px(FOOT)).child(canvas)
    }

    /// The row for `event`, the `line`th line of the panel.
    fn upcoming_row(
        &self,
        line: usize,
        model: &Model,
        event: usize,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        let theme = Theme::get(cx).clone();
        let bookmark = model.bookmark(event);
        let time = model.events[event].span.map(|s| labels::start_time(&s));
        let details = details(time, labels::place(bookmark), &theme);
        let row = self
            .event_row(("upcoming-event", line).into(), model, event, STYLE, cx)
            .gap_2p5()
            .child(site_mark(&bookmark.site, px(22.), cx))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .truncate()
                            .child(bookmark.title.clone()),
                    )
                    .child(details),
            );
        with_bookmark_menu(row, bookmark.id, cx).into_any_element()
    }
}

/// When it starts, and where.
fn details(time: Option<String>, place: String, theme: &Theme) -> impl IntoElement {
    div()
        .flex()
        .gap_1p5()
        .text_xs()
        .text_color(theme.muted)
        .when_some(time, |s, time| {
            s.child(div().flex_none().text_color(theme.text).child(time))
        })
        .child(div().min_w_0().truncate().child(place))
}

fn title() -> impl IntoElement {
    div()
        .flex_none()
        .h(px(TITLE_HEIGHT))
        .px_4()
        .flex()
        .items_center()
        .text_sm()
        .font_weight(FontWeight::SEMIBOLD)
        .child("Upcoming")
}
