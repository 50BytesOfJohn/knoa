//! How the Events views name days, months and times: English, 24-hour, like the
//! `datetime` fields' own display.

use jiff::civil::{Date, DateTime, Weekday};
use knoa_core::Bookmark;

use super::span::Span;

const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// "Today", "Tomorrow", a weekday within the week ahead, else "Sat 12 Dec", with the year
/// when it isn't this one.
pub fn day_heading(day: Date, today: Date) -> String {
    let ahead = (day - today).get_days();
    match ahead {
        0 => "Today".into(),
        1 => "Tomorrow".into(),
        2..=6 => weekday_name(day.weekday()).into(),
        _ if day.year() == today.year() => short_date(day),
        _ => format!("{} {}", short_date(day), day.year()),
    }
}

/// "December 2026".
pub fn month_heading(day: Date) -> String {
    format!("{} {}", month_name(day), day.year())
}

/// "Dec 2026".
pub fn short_month_heading(day: Date) -> String {
    format!("{} {}", short_month(day), day.year())
}

/// "Sat 12 Dec".
pub fn short_date(day: Date) -> String {
    format!(
        "{} {} {}",
        short_weekday(day.weekday()),
        day.day(),
        short_month(day)
    )
}

/// When it starts, as a row shows it: "19:00", or "All day".
pub fn start_time(span: &Span) -> String {
    if span.all_day {
        "All day".into()
    } else {
        clock(span.start.local)
    }
}

/// From start to end: "19:00 – 23:00", "5 – 7 Dec", "30 Nov 20:00 – 1 Dec 02:00". An
/// end shows only where the page gave one.
pub fn time_range(span: &Span) -> String {
    let (first, last) = (span.start.local.date(), span.last_day());
    let start = span.start.local;
    match (span.all_day, span.end) {
        (true, _) if first == last => "All day".into(),
        (true, _) => day_range(first, last),
        (false, None) => clock(start),
        (false, Some(end)) if first == last => format!("{} – {}", clock(start), clock(end.local)),
        (false, Some(end)) => format!(
            "{} {} – {} {}",
            day_month(first),
            clock(start),
            day_month(end.local.date()),
            clock(end.local)
        ),
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

pub fn short_weekday(day: Weekday) -> &'static str {
    &weekday_name(day)[..3]
}

pub fn short_month(day: Date) -> &'static str {
    &month_name(day)[..3]
}

fn day_range(first: Date, last: Date) -> String {
    if first.month() == last.month() && first.year() == last.year() {
        format!("{} – {}", first.day(), day_month(last))
    } else {
        format!("{} – {}", day_month(first), day_month(last))
    }
}

fn day_month(day: Date) -> String {
    format!("{} {}", day.day(), short_month(day))
}

fn clock(time: DateTime) -> String {
    format!("{:02}:{:02}", time.hour(), time.minute())
}

fn weekday_name(day: Weekday) -> &'static str {
    WEEKDAYS[usize::from(day.to_monday_zero_offset().unsigned_abs())]
}

fn month_name(day: Date) -> &'static str {
    MONTHS[usize::from(day.month().unsigned_abs()) - 1]
}
