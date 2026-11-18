//! When one event runs in this Mac's time zone. An offset-bearing time is an instant, so
//! order and "now" compare instants, which a repeated hour at a DST change can't confuse;
//! the local wall time is kept beside it to place the event on days and show it.

use jiff::{
    Timestamp, ToSpan as _,
    civil::{Date, DateTime, Time},
    tz::{Offset, TimeZone},
};
use knoa_core::{EventTime, Schedule};

/// The most days one event is shown on, so a season-long exhibition doesn't fill months
/// of the calendar.
const MAX_DAYS: usize = 60;

/// A point in an event's schedule. Ordered by instant; a floating time takes the instant
/// at which this Mac's clock shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Moment {
    pub at: Timestamp,
    pub local: DateTime,
}

/// When an event runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Moment,
    /// The end the page gave, exclusive. `None` when it gave none, none that parsed, or
    /// one before the start: views then show no end.
    pub end: Option<Moment>,
    /// When it is over: its end, or the end of the day it starts on.
    pub until: Moment,
    /// Only a date was given: the event fills its days.
    pub all_day: bool,
}

/// Where an event stands against the current time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Happening,
    Upcoming,
    Past,
}

impl Span {
    /// The event's span in `tz`, or `None` when its date can't be placed.
    pub fn of(schedule: &Schedule, tz: &TimeZone) -> Option<Span> {
        let start = Written::of(&schedule.starts)?.moment(tz)?;
        let end = schedule
            .ends
            .and_then(|ends| end_of(&ends, tz).filter(|end| reaches(&ends, end, &start)));
        let until = match end {
            Some(end) => end,
            None => next_midnight(start.local.date(), tz)?,
        };
        Some(Span {
            start,
            end,
            until,
            all_day: schedule.starts.time.is_none(),
        })
    }

    /// The local days it covers, at most [`MAX_DAYS`].
    pub fn days(&self) -> impl Iterator<Item = Date> {
        let last = self.last_day();
        self.start
            .local
            .date()
            .series(1.day())
            .take_while(move |day| *day <= last)
            .take(MAX_DAYS)
    }

    /// The last day it covers: an end at midnight belongs to the day before.
    pub fn last_day(&self) -> Date {
        let until = self.until.local;
        let last = if until.time() == Time::midnight() {
            until.date().yesterday().unwrap_or(until.date())
        } else {
            until.date()
        };
        last.max(self.start.local.date())
    }

    pub fn phase(&self, now: Timestamp) -> Phase {
        if now < self.start.at {
            Phase::Upcoming
        } else if now < self.until.at {
            Phase::Happening
        } else {
            Phase::Past
        }
    }
}

/// The wall time an event time shows in `tz`: an instant is converted, a floating time and
/// a date stay as written.
pub(in crate::ui::main) fn local_time(time: &EventTime, tz: &TimeZone) -> Option<DateTime> {
    match Written::of(time)? {
        Written::Instant(at) => Some(tz.to_datetime(at)),
        Written::Floating(wall) => Some(wall),
    }
}

/// An event time as the page gave it: with an offset, one instant; without, a wall time
/// (midnight for a date alone) that means the same on any clock.
enum Written {
    Instant(Timestamp),
    Floating(DateTime),
}

impl Written {
    fn of(time: &EventTime) -> Option<Written> {
        let date = Date::new(
            i16::try_from(time.date.year()).ok()?,
            i8::try_from(time.date.month()).ok()?,
            i8::try_from(time.date.day()).ok()?,
        )
        .ok()?;
        let Some(clock) = time.time else {
            return Some(Written::Floating(date.to_datetime(Time::midnight())));
        };
        let hour = i8::try_from(clock.hour).ok()?;
        let minute = i8::try_from(clock.minute).ok()?;
        let wall = date.at(hour, minute, 0, 0);
        let Some(minutes) = clock.offset_minutes else {
            return Some(Written::Floating(wall));
        };
        let offset = Offset::from_seconds(i32::from(minutes) * 60).ok()?;
        offset.to_timestamp(wall).ok().map(Written::Instant)
    }

    fn moment(self, tz: &TimeZone) -> Option<Moment> {
        match self {
            Written::Instant(at) => Some(Moment {
                at,
                local: tz.to_datetime(at),
            }),
            Written::Floating(wall) => wall_moment(wall, tz),
        }
    }
}

/// An end without a time lasts through that day.
/// Whether `end` is not before `start`. A date-only end is the midnight after its day,
/// so it must pass the start to cover the start's day.
fn reaches(ends: &EventTime, end: &Moment, start: &Moment) -> bool {
    if ends.time.is_none() {
        end.at > start.at
    } else {
        end.at >= start.at
    }
}

fn end_of(ends: &EventTime, tz: &TimeZone) -> Option<Moment> {
    match Written::of(ends)? {
        Written::Floating(day) if ends.time.is_none() => next_midnight(day.date(), tz),
        written => written.moment(tz),
    }
}

fn next_midnight(day: Date, tz: &TimeZone) -> Option<Moment> {
    wall_moment(day.tomorrow().ok()?.to_datetime(Time::midnight()), tz)
}

/// A wall time as this Mac's clock reaches it; in a skipped hour, as the clock jumps.
fn wall_moment(local: DateTime, tz: &TimeZone) -> Option<Moment> {
    let at = tz.to_ambiguous_timestamp(local).compatible().ok()?;
    Some(Moment { at, local })
}
