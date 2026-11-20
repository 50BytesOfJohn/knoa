//! Field values as the user reads them: `knoa_core`'s display, but dates, times and
//! durations in the app's language, and a time with an offset in this Mac's time zone,
//! where the Events page places it.

use jiff::{civil, tz::TimeZone};
use knoa_core::{Bookmark, EventTime, FieldDef, FieldType};
use serde_json::Value;

use super::events::local_time;
use crate::{format, i18n::t};

pub(super) fn display(def: &FieldDef, value: &Value) -> String {
    let shown = match def.ty {
        FieldType::DateTime => value.as_str().and_then(event_time),
        FieldType::Date => value.as_str().and_then(date),
        FieldType::Duration => value.as_i64().map(duration),
        _ => None,
    };
    shown.unwrap_or_else(|| def.display(value))
}

pub(super) fn key_facts(bookmark: &Bookmark) -> Vec<String> {
    bookmark.key_facts(display)
}

/// "Sat 5 Dec 2026, 19:00", or the date alone. `None` for text that isn't one, which
/// shows as written.
fn event_time(text: &str) -> Option<String> {
    let time = EventTime::parse(text)?;
    let local = local_time(&time, &TimeZone::system())?;
    let skeleton = if time.time.is_some() {
        format::WEEKDAY_DATE_YEAR_TIME
    } else {
        format::WEEKDAY_DATE_YEAR
    };
    Some(format::floating(local, skeleton))
}

/// `148` minutes as "2h 28m", `45` as "45 min".
fn duration(minutes: i64) -> String {
    match (minutes / 60, minutes % 60) {
        (0, minutes) => t!("main-duration-minutes", minutes = minutes),
        (hours, 0) => t!("main-duration-hours", hours = hours),
        (hours, minutes) => t!(
            "main-duration-hours-minutes",
            hours = hours,
            minutes = minutes
        ),
    }
}

/// A `YYYY-MM-DD` date, or only its month or year when that's all that is known.
fn date(text: &str) -> Option<String> {
    let mut parts = text.split('-').map(str::parse::<i8>);
    let year: i16 = text.get(..4)?.parse().ok()?;
    parts.next();
    match (parts.next(), parts.next()) {
        (None, _) => Some(year.to_string()),
        (Some(Ok(month)), None) => {
            let day = civil::Date::new(year, month, 1).ok()?;
            Some(format::day(day, format::MONTH_YEAR_SHORT))
        }
        (Some(Ok(month)), Some(Ok(day))) => {
            let day = civil::Date::new(year, month, day).ok()?;
            Some(format::day(day, format::DATE))
        }
        _ => None,
    }
}
