//! One event in the list: a tear-off date, its title, when and where, its site, and its
//! thumbnail when it has one.

use gpui::{
    AnyElement, Context, FontWeight, IntoElement, ObjectFit, ParentElement as _, Pixels,
    Styled as _, StyledImage as _, div, img, prelude::FluentBuilder as _, px,
};
use jiff::civil::Date;
use knoa_core::Bookmark;

use super::{labels, model::Model, rows::RowStyle};
use crate::{
    i18n::t,
    ui::{
        components::site_mark,
        main::{MainView, bookmark_menu::with_bookmark_menu},
        theme::Theme,
    },
};

/// The list's side padding, for its rows and the headings between them.
pub const INSET: Pixels = px(12.);
const STYLE: RowStyle = RowStyle {
    inset: INSET,
    past_opacity: 0.6,
};
const THUMBNAIL: f32 = 64.;

impl MainView {
    /// The row for `event`, the `line`th line of the list.
    pub(super) fn list_row(
        &self,
        line: usize,
        model: &Model,
        event: usize,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        let theme = Theme::get(cx).clone();
        let bookmark = model.bookmark(event);
        let span = model.events[event].span;
        let when = span.map_or_else(|| t!("events-date-not-set"), |s| labels::time_range(&s));
        let row = self
            .event_row(("event-row", line).into(), model, event, STYLE, cx)
            .gap_4()
            .child(date_tile(span.map(|s| s.start.local.date()), &theme))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(title(bookmark))
                    .child(when_and_where(when, labels::place(bookmark), &theme))
                    .child(site(bookmark, &theme, cx)),
            )
            .when_some(bookmark.image.as_ref(), |row, image| {
                row.child(
                    img(image.path.clone())
                        .flex_none()
                        .size(px(THUMBNAIL))
                        .rounded(px(6.))
                        .object_fit(ObjectFit::Cover),
                )
            });
        with_bookmark_menu(row, bookmark.id, cx).into_any_element()
    }
}

fn title(bookmark: &Bookmark) -> impl IntoElement {
    div()
        .text_size(px(14.))
        .font_weight(FontWeight::MEDIUM)
        .truncate()
        .child(bookmark.title.clone())
}

fn when_and_where(when: String, place: String, theme: &Theme) -> impl IntoElement {
    div()
        .flex()
        .gap_1p5()
        .text_xs()
        .text_color(theme.muted)
        .child(div().flex_none().text_color(theme.text).child(when))
        .when(!place.is_empty(), |s| {
            s.child(div().flex_none().text_color(theme.faint).child("·"))
                .child(div().min_w_0().truncate().child(place))
        })
}

fn site(bookmark: &Bookmark, theme: &Theme, cx: &gpui::App) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_1p5()
        .text_xs()
        .text_color(theme.muted)
        .child(site_mark(&bookmark.site, px(14.), cx))
        .child(div().truncate().child(bookmark.site.clone()))
}

/// A tear-off calendar page: weekday, day and month. A question mark for an undated event.
fn date_tile(day: Option<Date>, theme: &Theme) -> impl IntoElement {
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
    match day {
        Some(day) => tile.children(date_tile_page(day, theme)),
        None => tile.justify_center().text_color(theme.faint).child("?"),
    }
}

fn date_tile_page(day: Date, theme: &Theme) -> [AnyElement; 3] {
    let weekday = div()
        .w_full()
        .h(px(16.))
        .flex()
        .items_center()
        .justify_center()
        .bg(theme.accent)
        .text_color(theme.on_accent)
        .text_size(px(10.))
        .font_weight(FontWeight::SEMIBOLD)
        .child(labels::short_weekday(day.weekday()).to_uppercase());
    let number = div()
        .text_size(px(18.))
        .line_height(px(22.))
        .font_weight(FontWeight::SEMIBOLD)
        .child(day.day().to_string());
    let month = div()
        .text_size(px(10.))
        .text_color(theme.muted)
        .child(labels::short_month(day).to_uppercase());
    [
        weekday.into_any_element(),
        number.into_any_element(),
        month.into_any_element(),
    ]
}
