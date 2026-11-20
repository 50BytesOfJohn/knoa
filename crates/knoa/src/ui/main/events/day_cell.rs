//! One day of the month grid: its number, its first few events as chips, and how many
//! more there are. Too narrow for a readable chip, the day shows its events as dots.

use gpui::{
    AnyElement, ClickEvent, Context, Div, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, Stateful, StatefulInteractiveElement as _, Styled as _, div,
    prelude::FluentBuilder as _, px,
};
use jiff::civil::Date;

use super::{dots, labels, model::Model};
use crate::{
    i18n::t,
    ui::{
        main::{MainView, bookmark_menu::with_bookmark_menu},
        theme::Theme,
    },
};

const DAY_NUMBER: f32 = 26.;
const CHIP: f32 = 18.;
const CHIP_GAP: f32 = 2.;
/// The most chips a day shows before "+N more".
const MAX_CHIPS: usize = 3;
/// How faint a past event's chip or dot is.
pub const PAST_OPACITY: f32 = 0.55;
/// Narrower than this, a chip has no room for its time and a few letters of its title.
const MIN_CHIP_CELL: f32 = 90.;

/// How a day shows its events: as chips with their time and title, or as dots.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Density {
    Chips,
    Dots,
}

impl Density {
    pub fn for_cell_width(width: f32) -> Density {
        if width < MIN_CHIP_CELL {
            Density::Dots
        } else {
            Density::Chips
        }
    }
}

impl MainView {
    pub(super) fn day_cell(
        &self,
        model: &Model,
        day: Date,
        height: f32,
        density: Density,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        let theme = Theme::get(cx).clone();
        let page = &self.events;
        let in_month = day.year() == page.month.year() && day.month() == page.month.month();
        let events = model.on(day);
        let cell = div()
            .id(("day", day_id(day)))
            .flex_1()
            .min_w_0()
            .h_full()
            .px_1()
            .pb_1()
            .flex()
            .flex_col()
            .gap(px(CHIP_GAP))
            .overflow_hidden()
            .cursor_pointer()
            .when(page.day == Some(day), |s| s.bg(theme.selected_row()))
            .when(!in_month, |s| s.opacity(0.45))
            .child(day_number(day, day == model.today(), &theme))
            .on_click(cx.listener(move |this, _, _, cx| this.toggle_event_day(day, cx)));
        if density == Density::Dots {
            return cell
                .child(dots::dots(model, events, &theme))
                .into_any_element();
        }
        let shown = chips_shown(events.len(), height);
        let mut cell = cell;
        for &event in &events[..shown] {
            cell = cell.child(self.chip(model, event, day, cx));
        }
        cell.when(shown < events.len(), |cell| {
            cell.child(more(events.len() - shown, &theme))
        })
        .into_any_element()
    }

    /// One event on one day: its start time on the day it starts, then its title. A click
    /// picks the day too, so the keys move on through that day's events.
    fn chip(
        &self,
        model: &Model,
        event: usize,
        day: Date,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        let theme = Theme::get(cx);
        let bookmark = model.bookmark(event);
        let id = bookmark.id;
        let time = model.events[event]
            .span
            .filter(|s| !s.all_day && s.start.local.date() == day)
            .map(|s| labels::start_time(&s));
        let chip = chip_frame(event, self.selected == Some(id), theme)
            .when(model.is_past(event), |s| s.opacity(PAST_OPACITY))
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
                this.events.select_day(Some(day));
                this.click_card(id, event, window, cx);
            }));
        with_bookmark_menu(chip, id, cx).into_any_element()
    }
}

/// "+N more", for the events a day has no room for.
fn more(hidden: usize, theme: &Theme) -> impl IntoElement {
    div()
        .h(px(CHIP))
        .px_1p5()
        .text_xs()
        .text_color(theme.muted)
        .truncate()
        .child(t!("events-more", count = hidden))
}

fn chip_frame(event: usize, selected: bool, theme: &Theme) -> Stateful<Div> {
    let (bg, fg) = if selected {
        (theme.accent, theme.on_accent)
    } else {
        (theme.accent.opacity(0.12), theme.text)
    };
    let hover = theme.accent.opacity(0.2);
    div()
        .id(("chip", event))
        .h(px(CHIP))
        .px_1p5()
        .flex()
        .items_center()
        .gap_1()
        .overflow_hidden()
        .rounded(px(4.))
        .bg(bg)
        .text_color(fg)
        .text_xs()
        .when(!selected, |s| s.hover(move |s| s.bg(hover)))
}

/// How many chips fit in a cell `height` tall with `count` events, keeping a line for
/// "+N more" when some don't fit.
fn chips_shown(count: usize, height: f32) -> usize {
    let room = ((height - DAY_NUMBER - 4.) / (CHIP + CHIP_GAP))
        .floor()
        .max(0.) as usize;
    if count <= room.min(MAX_CHIPS) {
        count
    } else {
        room.min(MAX_CHIPS + 1).saturating_sub(1)
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

/// A day as a number for element ids: its count of days since jiff's first date.
fn day_id(day: Date) -> usize {
    usize::try_from((day - Date::MIN).get_days()).unwrap_or_default()
}
