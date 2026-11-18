//! The events among the results, placed in this Mac's time zone: when each one runs, the
//! days it covers, and whether it is on now, coming up or over. Built once per results
//! and minute, so drawing a frame only looks things up.

use std::{collections::BTreeMap, sync::Arc};

use jiff::{
    ToSpan as _,
    civil::{Date, DateTime, Time},
    tz::{Offset, TimeZone},
};
use knoa_core::{Bookmark, BookmarkId, EventTime, Schedule};

/// The most days one event is shown on, so a season-long exhibition doesn't fill months
/// of the calendar.
const MAX_DAYS: usize = 60;

/// When an event runs, in local wall time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: DateTime,
    /// The end the page gave, exclusive. `None` when it gave none.
    pub end: Option<DateTime>,
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

/// One event bookmark and when it runs. `span` is `None` when its date isn't set.
#[derive(Debug)]
pub struct Event {
    pub id: BookmarkId,
    /// Its place in the results.
    pub index: usize,
    pub span: Option<Span>,
}

/// The events, ordered and grouped the ways the views show them. Indices point into
/// `events`.
pub struct Model {
    pub results: Arc<Vec<Bookmark>>,
    pub events: Vec<Event>,
    pub now: DateTime,
    by_day: BTreeMap<Date, Vec<usize>>,
    pub happening: Vec<usize>,
    /// Events still to start, grouped by the local day they start on.
    pub upcoming: Vec<(Date, Vec<usize>)>,
    pub undated: Vec<usize>,
    /// Most recent first.
    pub past: Vec<usize>,
}

impl Span {
    /// The event's span in `tz`, or `None` when its date can't be placed.
    pub fn of(schedule: &Schedule, tz: &TimeZone) -> Option<Span> {
        let start = local(&schedule.starts, tz)?;
        let end = schedule
            .ends
            .and_then(|ends| end_of(&ends, tz))
            // A start and an end in different offsets can cross once both are local.
            .filter(|end| *end > start);
        Some(Span {
            start,
            end,
            all_day: schedule.starts.time.is_none(),
        })
    }

    /// When it is over: its end, or the end of the day it starts on.
    pub fn until(&self) -> DateTime {
        self.end.unwrap_or_else(|| next_midnight(self.start.date()))
    }

    /// The local days it covers, at most [`MAX_DAYS`].
    pub fn days(&self) -> impl Iterator<Item = Date> {
        let last = self.last_day();
        self.start
            .date()
            .series(1.day())
            .take_while(move |day| *day <= last)
            .take(MAX_DAYS)
    }

    /// The last day it covers: an end at midnight belongs to the day before.
    pub fn last_day(&self) -> Date {
        let until = self.until();
        let last = if until.time() == Time::midnight() {
            until.date().yesterday().unwrap_or(until.date())
        } else {
            until.date()
        };
        last.max(self.start.date())
    }

    pub fn phase(&self, now: DateTime) -> Phase {
        if now < self.start {
            Phase::Upcoming
        } else if now < self.until() {
            Phase::Happening
        } else {
            Phase::Past
        }
    }
}

impl Model {
    pub fn build(results: Arc<Vec<Bookmark>>, now: DateTime, tz: &TimeZone) -> Model {
        let mut events: Vec<Event> = results
            .iter()
            .enumerate()
            .map(|(index, bookmark)| Event {
                id: bookmark.id,
                index,
                span: Schedule::of(&bookmark.fields).and_then(|s| Span::of(&s, tz)),
            })
            .collect();
        // Undated last; on the same start, all-day first.
        events.sort_by_key(|e| e.span.map(|s| (s.start, !s.all_day)).ok_or(()));
        let mut model = Model {
            results,
            events,
            now,
            by_day: BTreeMap::new(),
            happening: Vec::new(),
            upcoming: Vec::new(),
            undated: Vec::new(),
            past: Vec::new(),
        };
        model.group();
        model
    }

    /// The events on `day`, all-day ones first, then by start.
    pub fn on(&self, day: Date) -> &[usize] {
        self.by_day.get(&day).map_or(&[], Vec::as_slice)
    }

    pub fn bookmark(&self, event: usize) -> &Bookmark {
        &self.results[self.events[event].index]
    }

    /// Whether `event` has ended.
    pub fn is_past(&self, event: usize) -> bool {
        self.events[event]
            .span
            .is_some_and(|span| span.phase(self.now) == Phase::Past)
    }

    pub fn today(&self) -> Date {
        self.now.date()
    }

    fn group(&mut self) {
        for (at, event) in self.events.iter().enumerate() {
            let Some(span) = event.span else {
                self.undated.push(at);
                continue;
            };
            for day in span.days() {
                self.by_day.entry(day).or_default().push(at);
            }
            match span.phase(self.now) {
                Phase::Happening => self.happening.push(at),
                Phase::Past => self.past.push(at),
                Phase::Upcoming => match self.upcoming.last_mut() {
                    Some((day, list)) if *day == span.start.date() => list.push(at),
                    _ => self.upcoming.push((span.start.date(), vec![at])),
                },
            }
        }
        self.past.reverse();
    }
}

/// The wall time an event time shows in `tz`: an instant is converted, a floating time and
/// a date stay as written.
fn local(time: &EventTime, tz: &TimeZone) -> Option<DateTime> {
    let date = civil_date(time.date)?;
    let Some(clock) = time.time else {
        return Some(date.to_datetime(Time::midnight()));
    };
    let wall = date.at(clock.hour as i8, clock.minute as i8, 0, 0);
    match clock.offset_minutes {
        None => Some(wall),
        Some(minutes) => {
            let offset = Offset::from_seconds(i32::from(minutes) * 60).ok()?;
            let instant = offset.to_timestamp(wall).ok()?;
            Some(tz.to_datetime(instant))
        }
    }
}

/// An end without a time lasts through that day.
fn end_of(ends: &EventTime, tz: &TimeZone) -> Option<DateTime> {
    match ends.time {
        None => Some(next_midnight(civil_date(ends.date)?)),
        Some(_) => local(ends, tz),
    }
}

fn next_midnight(day: Date) -> DateTime {
    day.tomorrow()
        .map_or(DateTime::MAX, |next| next.to_datetime(Time::midnight()))
}

fn civil_date(date: knoa_core::Date) -> Option<Date> {
    let year = i16::try_from(date.year()).ok()?;
    let month = i8::try_from(date.month()).ok()?;
    let day = i8::try_from(date.day()).ok()?;
    Date::new(year, month, day).ok()
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;
    use knoa_core::{BookmarkId, Status, WorkspaceId};
    use serde_json::json;

    use super::*;

    fn event(id: i64, starts: &str, ends: Option<&str>) -> Bookmark {
        let mut bookmark = undated(id);
        bookmark.fields.insert("starts".into(), json!(starts));
        if let Some(ends) = ends {
            bookmark.fields.insert("ends".into(), json!(ends));
        }
        bookmark
    }

    fn undated(id: i64) -> Bookmark {
        Bookmark {
            id: BookmarkId(id),
            workspace: WorkspaceId(1),
            url: format!("https://example.com/{id}"),
            title: format!("Event {id}"),
            site: "example.com".into(),
            added_at: 0,
            kind: Some(knoa_core::Kind::Event),
            summary: None,
            fields: knoa_core::Fields::new(),
            extras: Vec::new(),
            tags: Vec::new(),
            folders: Vec::new(),
            up_next: false,
            status: Status::Organized,
            image: None,
            browser_folder: None,
            removed_from_browser: false,
            set_aside: None,
            deleted_at: None,
            model: None,
        }
    }

    fn build(events: Vec<Bookmark>, now: DateTime) -> Model {
        let tz = TimeZone::get("Europe/Warsaw").expect("bundled tz database");
        Model::build(Arc::new(events), now, &tz)
    }

    #[test]
    fn an_instant_lands_on_the_viewers_local_day() {
        // 23:30 in New York is 05:30 the next morning in Warsaw.
        let model = build(
            vec![event(1, "2026-12-04T23:30-05:00", None)],
            date(2026, 12, 1).at(9, 0, 0, 0),
        );
        assert_eq!(model.on(date(2026, 12, 5)), [0]);
        assert_eq!(model.on(date(2026, 12, 4)), [0_usize; 0]);
    }

    #[test]
    fn a_multi_day_event_shows_on_each_day_and_is_happening_until_it_ends() {
        let model = build(
            vec![event(1, "2026-11-30", Some("2026-12-02"))],
            date(2026, 12, 2).at(23, 0, 0, 0),
        );
        for day in [date(2026, 11, 30), date(2026, 12, 1), date(2026, 12, 2)] {
            assert_eq!(model.on(day), [0], "{day}");
        }
        assert_eq!(model.on(date(2026, 12, 3)), [0_usize; 0]);
        assert_eq!(model.happening, [0]);
    }

    #[test]
    fn ended_events_are_past_most_recent_first_and_undated_stay_apart() {
        let model = build(
            vec![
                event(1, "2026-11-01T19:00", None),
                event(2, "2026-11-10T19:00", None),
                undated(3),
                event(4, "2026-12-24", None),
            ],
            date(2026, 11, 18).at(12, 0, 0, 0),
        );
        let ids =
            |list: &[usize]| -> Vec<i64> { list.iter().map(|&i| model.events[i].id.0).collect() };
        assert_eq!(ids(&model.past), [2, 1]);
        assert_eq!(ids(&model.undated), [3]);
        assert_eq!(model.upcoming.len(), 1);
        assert_eq!(ids(&model.upcoming[0].1), [4]);
    }
}
