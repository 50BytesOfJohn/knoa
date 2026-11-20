//! How the Events views name days, months and times, in the app's language.

use jiff::{
    ToSpan as _,
    civil::{Date, DateTime, Weekday, date},
};
use knoa_core::Bookmark;

use super::span::Span;
use crate::{format, i18n::t};

/// "Today", "Tomorrow", a weekday within the week ahead, else "Sat 12 Dec", with the year
/// when it isn't this one.
pub fn day_heading(day: Date, today: Date) -> String {
    let ahead = (day - today).get_days();
    match ahead {
        0 => t!("date-today"),
        1 => t!("date-tomorrow"),
        2..=6 => format::day(day, format::WEEKDAY),
        _ if day.year() == today.year() => short_date(day),
        _ => format::day(day, format::WEEKDAY_DATE_YEAR),
    }
}

/// "December 2026".
pub fn month_heading(day: Date) -> String {
    format::day(day, format::MONTH_YEAR)
}

/// "Dec", or "Dec 2027" when it isn't this year.
pub fn short_month_heading(day: Date, today: Date) -> String {
    if day.year() == today.year() {
        short_month(day)
    } else {
        format::day(day, format::MONTH_YEAR_SHORT)
    }
}

/// "Sat 12 Dec".
pub fn short_date(day: Date) -> String {
    format::day(day, format::WEEKDAY_DATE)
}

/// When it starts, as a row shows it: "19:00", or "All day".
pub fn start_time(span: &Span) -> String {
    if span.all_day {
        t!("date-all-day")
    } else {
        format::floating(span.start.local, format::TIME)
    }
}

/// From start to end: "19:00 – 23:00", "5 – 7 Dec", "30 Nov, 20:00 – 1 Dec, 02:00". An
/// end shows only where the page gave one.
pub fn time_range(span: &Span) -> String {
    let (first, last) = (span.start.local.date(), span.last_day());
    let start = span.start.local;
    match (span.all_day, span.end) {
        (true, _) if first == last => t!("date-all-day"),
        (true, _) => format::floating_range(midnight(first), midnight(last), format::DAY_MONTH),
        (false, None) => format::floating(start, format::TIME),
        (false, Some(end)) if first == last => {
            format::floating_range(start, end.local, format::TIME)
        }
        (false, Some(end)) => format::floating_range(start, end.local, format::DAY_MONTH_TIME),
    }
}

/// "Venue · City", from what the event has of them.
pub fn place(bookmark: &Bookmark) -> String {
    ["venue", "city"]
        .iter()
        .filter_map(|name| bookmark.fields.get(*name)?.as_str())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

/// "Mon", as over a calendar's column.
pub fn short_weekday(weekday: Weekday) -> String {
    // 3 January 2000 was a Monday.
    let monday = date(2000, 1, 3);
    let offset = i64::from(weekday.to_monday_zero_offset());
    format::day(monday.saturating_add(offset.days()), format::WEEKDAY_SHORT)
}

pub fn short_month(day: Date) -> String {
    format::day(day, format::MONTH_SHORT)
}

fn midnight(day: Date) -> DateTime {
    day.to_datetime(jiff::civil::Time::midnight())
}
