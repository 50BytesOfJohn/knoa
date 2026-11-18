//! The Upcoming panel beside the calendar: the clicked day's events, then what is on now
//! and what is coming, by day, then the events with no date.

use gpui::{
    AnyElement, ClickEvent, Context, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _, div,
    prelude::FluentBuilder as _, px,
};
use jiff::civil::Date;

use super::{
    labels,
    lines::{Heights, Line, Lines},
    model::Model,
};
use crate::ui::{
    components::site_mark,
    main::{MainView, bookmark_menu::with_bookmark_menu},
    theme::{PANEL_RADIUS, ROW_RADIUS, Theme},
};

pub const WIDTH: f32 = 300.;
const TITLE_HEIGHT: f32 = 44.;
const HEIGHTS: Heights = Heights {
    month: 0.,
    heading: 30.,
    event: 54.,
    note: 60.,
};
/// Rows built above and below the panel's view.
const OVERSCAN: f32 = 300.;

/// The panel's lines for `model`, with `day`'s events first when a day is picked.
pub fn lines(model: &Model, day: Option<Date>) -> Lines {
    let today = model.today();
    let mut lines = Lines::default();
    let section = |lines: &mut Lines, heading: String, events: &[usize]| {
        lines.push(Line::Heading(heading.into()), &HEIGHTS);
        for &event in events {
            lines.push(Line::Event(event), &HEIGHTS);
        }
    };
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
        lines.push(
            Line::Note("Nothing coming up. Events you save show here until they end.".into()),
            &HEIGHTS,
        );
    }
    if !model.undated.is_empty() {
        section(&mut lines, "Date not set".into(), &model.undated);
    }
    lines
}

impl MainView {
    pub(super) fn upcoming_panel(&self, height: f32, cx: &mut Context<'_, MainView>) -> AnyElement {
        let theme = Theme::get(cx).clone();
        let page = &self.events;
        let Some(model) = &page.model else {
            return div().into_any_element();
        };
        let top = -f32::from(page.upcoming_scroll.offset().y);
        let view_height = height - TITLE_HEIGHT;
        let mut canvas = div().relative().w_full().h(px(page.upcoming.height));
        for placed in page
            .upcoming
            .visible(top - OVERSCAN, top + view_height + OVERSCAN)
        {
            let line = match &placed.line {
                Line::Heading(text) => heading(text.clone(), &theme).into_any_element(),
                Line::Note(text) => note(text.clone(), &theme).into_any_element(),
                Line::Event(event) => self.upcoming_row(model, *event, &theme, cx),
                Line::Month(_) => continue,
            };
            canvas = canvas.child(
                div()
                    .absolute()
                    .top(px(placed.y))
                    .left_0()
                    .right_0()
                    .h(px(placed.height))
                    .child(line),
            );
        }
        div()
            .flex_none()
            .w(px(WIDTH))
            .h(px(height))
            .flex()
            .flex_col()
            .rounded(PANEL_RADIUS)
            .bg(theme.panel)
            .border_1()
            .border_color(theme.hairline)
            .overflow_hidden()
            .child(
                div()
                    .flex_none()
                    .h(px(TITLE_HEIGHT))
                    .px_4()
                    .flex()
                    .items_center()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Upcoming"),
            )
            .child(
                div()
                    .id("upcoming")
                    .flex_1()
                    .overflow_y_scroll()
                    .track_scroll(&page.upcoming_scroll)
                    .on_scroll(cx.listener(|_, _, _, cx| cx.notify()))
                    .px_2()
                    .child(canvas)
                    .child(div().h(px(8.))),
            )
            .into_any_element()
    }

    fn upcoming_row(
        &self,
        model: &Model,
        event: usize,
        theme: &Theme,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        let bookmark = model.bookmark(event);
        let id = bookmark.id;
        let span = model.events[event].span;
        let past = model.is_past(event);
        let time = span.map_or_else(String::new, |s| labels::start_time(&s));
        let place = labels::place(bookmark);
        let selected = self.selected == Some(id);
        let hover = theme.hover;
        let row = div()
            .id(("upcoming-event", event))
            .size_full()
            .px_2()
            .flex()
            .items_center()
            .gap_2p5()
            .rounded(ROW_RADIUS)
            .cursor_pointer()
            .when(selected, |s| s.bg(theme.selected_row()))
            .when(!selected, |s| s.hover(move |s| s.bg(hover)))
            .when(past, |s| s.opacity(0.55))
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
                    .child(
                        div()
                            .flex()
                            .gap_1p5()
                            .text_xs()
                            .text_color(theme.muted)
                            .when(!time.is_empty(), |s| {
                                s.child(div().flex_none().text_color(theme.text).child(time))
                            })
                            .child(div().min_w_0().truncate().child(place)),
                    ),
            )
            .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                this.click_card(id, event, window, cx);
            }))
            .on_aux_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                this.click_card(id, event, window, cx);
            }));
        with_bookmark_menu(row, id, cx).into_any_element()
    }
}

fn heading(text: SharedString, theme: &Theme) -> impl IntoElement {
    div()
        .size_full()
        .px_2()
        .pt_2()
        .flex()
        .items_center()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme.muted)
        .child(text)
}

fn note(text: SharedString, theme: &Theme) -> impl IntoElement {
    div()
        .size_full()
        .px_2()
        .py_2()
        .text_sm()
        .text_color(theme.muted)
        .child(text)
}
