//! The list of events: on now, then coming up by day under month names, then those with
//! no date, then the past, most recent first.

use std::{collections::HashSet, path::PathBuf};

use gpui::{
    AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use super::{
    labels,
    lines::{Heights, Line, Lines},
    list_row::INSET,
    model::Model,
    rows,
};
use crate::ui::{
    main::{MainView, content::KEEP_BEYOND},
    theme::Theme,
};

const HEIGHTS: Heights = Heights {
    month: 52.,
    heading: 34.,
    event: 80.,
    note: 44.,
};
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
        for (index, placed) in lines.visible(top - OVERSCAN, top + height + OVERSCAN) {
            let line = match placed.line {
                Line::Event(event) => self.list_row(index, model, event, cx),
                _ => rows::text_line(&placed.line, INSET, &theme),
            };
            canvas = canvas.child(rows::placed(placed, line));
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
                .filter_map(|(_, placed)| match placed.line {
                    Line::Event(event) => model.bookmark(event).image.as_ref(),
                    _ => None,
                })
                .map(|image| image.path.clone())
        };
        let built: Vec<PathBuf> = images(top - OVERSCAN, bottom + OVERSCAN).collect();
        let near: HashSet<PathBuf> = images(top - KEEP_BEYOND, bottom + KEEP_BEYOND).collect();
        self.keep_decoded(built, near, window, cx);
    }
}
