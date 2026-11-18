//! How the Events views name days, months and times: English, 24-hour, like the
//! `datetime` fields' own display.

use jiff::civil::{Date, DateTime, Weekday};
use knoa_core::Bookmark;

use super::model::Span;

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
        clock(span.start)
    }
}

/// From start to end: "19:00 – 23:00", "5 – 7 Dec", "30 Nov 20:00 – 1 Dec 02:00".
pub fn time_range(span: &Span) -> String {
    let (first, last) = (span.start.date(), span.last_day());
    match (span.all_day, first == last) {
        (true, true) => "All day".into(),
        (true, false) => day_range(first, last),
        (false, true) => match span.end {
            Some(end) => format!("{} – {}", clock(span.start), clock(end)),
            None => clock(span.start),
        },
        (false, false) => {
            let end = span.until();
            format!(
                "{} {} – {} {}",
                day_month(first),
                clock(span.start),
                day_month(end.date()),
                clock(end)
            )
        }
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
    WEEKDAYS[day.to_monday_zero_offset() as usize]
}

fn month_name(day: Date) -> &'static str {
    MONTHS[(day.month() - 1) as usize]
}
