//! The list of events: on now, then coming up by day under month names, then those with
//! no date, then the past, most recent first.

use std::{collections::HashSet, path::PathBuf};

use gpui::{
    AnyElement, ClickEvent, Context, FontWeight, InteractiveElement as _, IntoElement, ObjectFit,
    ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _,
    StyledImage as _, Window, div, img, prelude::FluentBuilder as _, px,
};

use super::{
    labels,
    lines::{Heights, Line, Lines},
    model::Model,
};
use crate::ui::{
    components::site_mark,
    main::{MainView, bookmark_menu::with_bookmark_menu, content::KEEP_BEYOND},
    theme::{ROW_RADIUS, SERIF, Theme},
};

const HEIGHTS: Heights = Heights {
    month: 52.,
    heading: 34.,
    event: 80.,
    note: 44.,
};
const THUMBNAIL: f32 = 64.;
/// Rows built above and below the view, so fast scrolling doesn't show gaps.
const OVERSCAN: f32 = 400.;

pub fn lines(model: &Model) -> Lines {
    let today = model.today();
    let mut lines = Lines::default();
    let rows = |lines: &mut Lines, events: &[usize]| {
        for &event in events {
            lines.push(Line::Event(event), &HEIGHTS);
        }
    };
    if !model.happening.is_empty() {
        lines.push(Line::Heading("Happening now".into()), &HEIGHTS);
        rows(&mut lines, &model.happening);
    }
    let mut month = None;
    for (day, events) in &model.upcoming {
        let this_month = (day.year(), day.month());
        if month != Some(this_month) {
            month = Some(this_month);
            lines.push(Line::Month(labels::month_heading(*day).into()), &HEIGHTS);
        }
        lines.push(
            Line::Heading(labels::day_heading(*day, today).into()),
            &HEIGHTS,
        );
        rows(&mut lines, events);
    }
    if model.happening.is_empty() && model.upcoming.is_empty() {
        lines.push(Line::Note("Nothing coming up.".into()), &HEIGHTS);
    }
    if !model.undated.is_empty() {
        lines.push(Line::Heading("Date not set".into()), &HEIGHTS);
        rows(&mut lines, &model.undated);
    }
    if !model.past.is_empty() {
        lines.push(Line::Month("Past".into()), &HEIGHTS);
        rows(&mut lines, &model.past);
    }
    lines
}

impl MainView {
    pub(super) fn events_list(
        &mut self,
        height: f32,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        let top = -f32::from(self.scroll.offset().y);
        self.keep_list_images(top, top + height, window, cx);
        let theme = Theme::get(cx).clone();
        let Some(model) = &self.events.model else {
            return div().into_any_element();
        };
        let lines = &self.events.list;
        let mut canvas = div().relative().w_full().h(px(lines.height));
        for placed in lines.visible(top - OVERSCAN, top + height + OVERSCAN) {
            let line = match &placed.line {
                Line::Month(text) => month(text.clone(), &theme).into_any_element(),
                Line::Heading(text) => heading(text.clone(), &theme).into_any_element(),
                Line::Note(text) => note(text.clone(), &theme).into_any_element(),
                Line::Event(event) => self.list_row(model, *event, &theme, cx),
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
            .id("events-list")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .on_scroll(cx.listener(|_, _, _, cx| cx.notify()))
            .child(canvas)
            .into_any_element()
    }

    /// Records the thumbnails of the rows built this frame and lets go of those far away.
    fn keep_list_images(
        &mut self,
        top: f32,
        bottom: f32,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        let page = &self.events;
        let Some(model) = &page.model else {
            return;
        };
        let images = |from: f32, to: f32| {
            page.list
                .visible(from, to)
                .filter_map(|placed| match placed.line {
                    Line::Event(event) => model.bookmark(event).image.as_ref(),
                    _ => None,
                })
                .map(|image| image.path.clone())
        };
        let built: Vec<PathBuf> = images(top - OVERSCAN, bottom + OVERSCAN).collect();
        let near: HashSet<PathBuf> = images(top - KEEP_BEYOND, bottom + KEEP_BEYOND).collect();
        self.keep_decoded(built, near, window, cx);
    }

    fn list_row(
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
        let selected = self.selected == Some(id);
        let when = span.map_or_else(|| "Date not set".to_owned(), |s| labels::time_range(&s));
        let place = labels::place(bookmark);
        let hover = theme.hover;
        let row = div()
            .id(("event-row", event))
            .size_full()
            .px_3()
            .flex()
            .items_center()
            .gap_4()
            .rounded(ROW_RADIUS)
            .cursor_pointer()
            .when(selected, |s| s.bg(theme.selected_row()))
            .when(!selected, |s| s.hover(move |s| s.bg(hover)))
            .when(past, |s| s.opacity(0.6))
            .child(date_tile(span.map(|s| s.start.date()), theme))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(
                        div()
                            .text_size(px(14.))
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
                            .child(div().flex_none().text_color(theme.text).child(when))
                            .when(!place.is_empty(), |s| {
                                s.child(div().flex_none().text_color(theme.faint).child("·"))
                                    .child(div().min_w_0().truncate().child(place))
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .text_xs()
                            .text_color(theme.muted)
                            .child(site_mark(&bookmark.site, px(14.), cx))
                            .child(div().truncate().child(bookmark.site.clone())),
                    ),
            )
            .when_some(bookmark.image.as_ref(), |row, image| {
                row.child(
                    img(image.path.clone())
                        .flex_none()
                        .size(px(THUMBNAIL))
                        .rounded(px(6.))
                        .object_fit(ObjectFit::Cover),
                )
            })
            .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                this.click_card(id, event, window, cx);
            }))
            .on_aux_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                this.click_card(id, event, window, cx);
            }));
        with_bookmark_menu(row, id, cx).into_any_element()
    }
}

/// A tear-off calendar page: weekday, day and month. Blank for an undated event.
fn date_tile(day: Option<jiff::civil::Date>, theme: &Theme) -> impl IntoElement {
    let tile = div()
        .flex_none()
        .w(px(48.))
        .h(px(56.))
        .flex()
        .flex_col()
        .items_center()
        .rounded(px(8.))
        .bg(theme.card)
        .border_1()
        .border_color(theme.hairline)
        .overflow_hidden();
    let Some(day) = day else {
        return tile.justify_center().text_color(theme.faint).child("?");
    };
    let caps = |text: &str| text.to_uppercase();
    tile.child(
        div()
            .w_full()
            .h(px(16.))
            .flex()
            .items_center()
            .justify_center()
            .bg(theme.accent)
            .text_color(theme.on_accent)
            .text_size(px(10.))
            .font_weight(FontWeight::SEMIBOLD)
            .child(caps(labels::short_weekday(day.weekday()))),
    )
    .child(
        div()
            .text_size(px(18.))
            .line_height(px(22.))
            .font_weight(FontWeight::SEMIBOLD)
            .child(day.day().to_string()),
    )
    .child(
        div()
            .text_size(px(10.))
            .text_color(theme.muted)
            .child(caps(labels::short_month(day))),
    )
}

fn month(text: SharedString, theme: &Theme) -> impl IntoElement {
    div()
        .size_full()
        .px_3()
        .pt_4()
        .flex()
        .items_end()
        .pb_1()
        .font_family(SERIF)
        .text_xl()
        .text_color(theme.text)
        .child(text)
}

fn heading(text: SharedString, theme: &Theme) -> impl IntoElement {
    div()
        .size_full()
        .px_3()
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
        .px_3()
        .flex()
        .items_center()
        .text_sm()
        .text_color(theme.muted)
        .child(text)
}
