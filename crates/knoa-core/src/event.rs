//! When an event happens: its `starts` and `ends` field values, read without a time-zone
//! database so this compiles to WebAssembly. Views convert to local time themselves.

use std::{cmp::Ordering, fmt};

use crate::Fields;

/// A calendar date, valid by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

/// A wall-clock time, and the UTC offset it was given in, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimeOfDay {
    pub hour: u8,
    pub minute: u8,
    /// Minutes east of UTC. Without one the time is floating: the same wall time anywhere.
    pub offset_minutes: Option<i16>,
}

/// One `datetime` field value. Without a time it is all-day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventTime {
    pub date: Date,
    pub time: Option<TimeOfDay>,
}

/// An event's start, and its end when it is known and not before the start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Schedule {
    pub starts: EventTime,
    pub ends: Option<EventTime>,
}

const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
/// Real offsets run from -12:00 to +14:00.
const MAX_OFFSET_MINUTES: i16 = 14 * 60;

impl Date {
    /// `None` unless the date exists, in years 1 to 9999.
    pub fn new(year: u16, month: u8, day: u8) -> Option<Date> {
        let valid = (1..=9999).contains(&year)
            && (1..=12).contains(&month)
            && (1..=days_in_month(year, month)).contains(&day);
        valid.then_some(Date { year, month, day })
    }

    pub fn year(self) -> u16 {
        self.year
    }

    pub fn month(self) -> u8 {
        self.month
    }

    pub fn day(self) -> u8 {
        self.day
    }

    /// Days since 1970-01-01, negative before it.
    pub fn days_since_epoch(self) -> i64 {
        // Howard Hinnant's days_from_civil.
        let year = i64::from(self.year) - i64::from(self.month <= 2);
        let era = year.div_euclid(400);
        let year_of_era = year - era * 400;
        let month = i64::from(self.month);
        let day_of_year =
            (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }

    /// 0 for Monday through 6 for Sunday.
    pub fn weekday(self) -> u8 {
        // 1970-01-01 was a Thursday.
        (self.days_since_epoch() + 3).rem_euclid(7) as u8
    }

    fn display(self) -> String {
        format!(
            "{} {} {} {}",
            WEEKDAYS[usize::from(self.weekday())],
            self.day,
            MONTHS[usize::from(self.month - 1)],
            self.year
        )
    }
}

/// `YYYY-MM-DD`.
impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl EventTime {
    /// Reads the stored forms (`2026-12-05`, `2026-12-05T19:00`, `2026-12-05T19:00+01:00`)
    /// and the ISO 8601 variants pages and models write: seconds, fractions, `Z`, `+0100`,
    /// `+01`, a space for `T`, and a trailing `[Europe/Paris]`. Nothing else is guessed.
    pub fn parse(text: &str) -> Option<EventTime> {
        let text = text.trim();
        let text = match text.find('[') {
            Some(bracket) if text.ends_with(']') => &text[..bracket],
            _ => text,
        };
        let date = parse_date(text.get(..10)?)?;
        let rest = &text[10..];
        if rest.is_empty() {
            return Some(EventTime { date, time: None });
        }
        let rest = rest
            .strip_prefix(['T', 't', ' '])
            .filter(|r| !r.starts_with(' '))?;
        Some(EventTime {
            date,
            time: Some(parse_time(rest)?),
        })
    }

    /// `Sat 5 Dec 2026, 19:00`, or `Sat 5 Dec 2026` without a time, in the value's own
    /// wall time.
    pub fn display(&self) -> String {
        match self.time {
            Some(time) => format!(
                "{}, {:02}:{:02}",
                self.date.display(),
                time.hour,
                time.minute
            ),
            None => self.date.display(),
        }
    }

    /// Minutes since the epoch at this wall time, ignoring the offset. With an offset,
    /// subtracting it gives the instant.
    fn wall_minutes(&self) -> i64 {
        let (hour, minute) = self
            .time
            .map_or((0, 0), |t| (i64::from(t.hour), i64::from(t.minute)));
        self.date.days_since_epoch() * 1440 + hour * 60 + minute
    }

    fn offset(&self) -> Option<i16> {
        self.time.and_then(|t| t.offset_minutes)
    }
}

/// The canonical stored form: `2026-12-05`, `2026-12-05T19:00` or `2026-12-05T19:00+01:00`.
impl fmt::Display for EventTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.date)?;
        let Some(time) = self.time else {
            return Ok(());
        };
        write!(f, "T{:02}:{:02}", time.hour, time.minute)?;
        if let Some(offset) = time.offset_minutes {
            let sign = if offset < 0 { '-' } else { '+' };
            let offset = offset.unsigned_abs();
            write!(f, "{sign}{:02}:{:02}", offset / 60, offset % 60)?;
        }
        Ok(())
    }
}

impl Schedule {
    /// The event's schedule from its `starts` and `ends` fields, or `None` when `starts`
    /// is missing or unreadable: the event's date isn't set.
    pub fn of(fields: &Fields) -> Option<Schedule> {
        let read = |name: &str| fields.get(name)?.as_str().and_then(EventTime::parse);
        let starts = read("starts")?;
        let ends = read("ends").filter(|ends| compare(ends, &starts) != Ordering::Less);
        Some(Schedule { starts, ends })
    }
}

/// Compares instants when both have offsets, wall times otherwise: without a time zone
/// that is the best a floating time allows.
fn compare(a: &EventTime, b: &EventTime) -> Ordering {
    match (a.offset(), b.offset()) {
        (Some(x), Some(y)) => {
            (a.wall_minutes() - i64::from(x)).cmp(&(b.wall_minutes() - i64::from(y)))
        }
        _ => a.wall_minutes().cmp(&b.wall_minutes()),
    }
}

fn parse_date(text: &str) -> Option<Date> {
    let bytes = text.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    Date::new(
        digits(&text[..4])?,
        digits(&text[5..7])?,
        digits(&text[8..])?,
    )
}

/// `HH:MM`, optional `:SS` and fraction, then nothing, `Z`, or an offset.
fn parse_time(text: &str) -> Option<TimeOfDay> {
    let hour: u8 = digits(text.get(..2)?)?;
    let minute: u8 = digits(text.get(3..5).filter(|_| text.get(2..3) == Some(":"))?)?;
    let mut rest = &text[5..];
    if let Some(seconds) = rest.strip_prefix(':') {
        digits::<u8>(seconds.get(..2)?).filter(|s| *s <= 60)?;
        rest = &seconds[2..];
        if let Some(fraction) = rest.strip_prefix(['.', ',']) {
            let end = fraction
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(fraction.len());
            if end == 0 {
                return None;
            }
            rest = &fraction[end..];
        }
    }
    let rest = rest.trim_start();
    let offset_minutes = if rest.is_empty() {
        None
    } else {
        Some(parse_offset(rest)?)
    };
    (hour <= 23 && minute <= 59).then_some(TimeOfDay {
        hour,
        minute,
        offset_minutes,
    })
}

/// `Z`, `+01:00`, `+0100` or `+01` as minutes east of UTC.
fn parse_offset(text: &str) -> Option<i16> {
    if text.eq_ignore_ascii_case("z") {
        return Some(0);
    }
    if !text.is_ascii() {
        return None;
    }
    let sign: i16 = match text.as_bytes()[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let body = &text[1..];
    let (hours, minutes) = match body.len() {
        2 => (body, "00"),
        4 => body.split_at(2),
        5 if body.as_bytes()[2] == b':' => (&body[..2], &body[3..]),
        _ => return None,
    };
    let (hours, minutes): (i16, i16) = (digits(hours)?, digits(minutes)?);
    let total = hours * 60 + minutes;
    (minutes < 60 && total <= MAX_OFFSET_MINUTES).then_some(sign * total)
}

fn digits<T: std::str::FromStr>(text: &str) -> Option<T> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn iso_variants_read_as_their_canonical_form() {
        for (input, canonical) in [
            ("2026-12-05", "2026-12-05"),
            ("2026-12-05T19:00", "2026-12-05T19:00"),
            ("2026-12-05 19:00:30.250", "2026-12-05T19:00"),
            ("2026-12-05T18:00:00Z", "2026-12-05T18:00+00:00"),
            ("2026-12-05T19:00+0100", "2026-12-05T19:00+01:00"),
            ("2026-12-05T13:00-05", "2026-12-05T13:00-05:00"),
            (
                "2026-12-05T19:00:00+01:00[Europe/Paris]",
                "2026-12-05T19:00+01:00",
            ),
        ] {
            let time = EventTime::parse(input).map(|t| t.to_string());
            assert_eq!(time.as_deref(), Some(canonical), "{input}");
        }
    }

    #[test]
    fn impossible_or_loose_times_are_refused() {
        for input in [
            "Dec 5",
            "2026-02-29",
            "2026-13-01",
            "2026-12-05T24:00",
            "2026-12-05T19:60",
            "2026-12-05T19:00+15:00",
            "2026-12-05T7pm",
            "2026-12-5",
        ] {
            assert_eq!(EventTime::parse(input), None, "{input}");
        }
    }

    #[test]
    fn display_names_the_weekday_in_the_values_own_wall_time() {
        let shown = |text| EventTime::parse(text).map(|t| t.display());
        assert_eq!(
            shown("2026-12-05T19:00-08:00").as_deref(),
            Some("Sat 5 Dec 2026, 19:00")
        );
        assert_eq!(shown("2024-02-29").as_deref(), Some("Thu 29 Feb 2024"));
    }

    #[test]
    fn an_end_before_the_start_instant_is_ignored() {
        let schedule = |ends| {
            let fields = json!({"starts": "2026-12-05T19:00+01:00", "ends": ends});
            Schedule::of(fields.as_object().unwrap()).unwrap().ends
        };
        // 19:30 in New York is after 19:00 in Paris, though the wall time reads earlier.
        assert!(schedule("2026-12-05T13:30-05:00").is_some());
        assert_eq!(schedule("2026-12-05T18:30+01:00"), None);
    }
}
