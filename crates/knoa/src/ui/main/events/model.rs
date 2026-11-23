//! The events among the results, placed in this Mac's time zone: the days each covers,
//! and whether it is on now, coming up or over. Built once per results and minute, so
//! drawing a frame only looks things up.

use std::{collections::BTreeMap, sync::Arc};

use jiff::{Timestamp, Zoned, civil::Date};
use knoa_core::{Bookmark, BookmarkId, Schedule};

use super::span::{Phase, Span};

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
    now: Timestamp,
    today: Date,
    by_day: BTreeMap<Date, Vec<usize>>,
    pub happening: Vec<usize>,
    /// Events still to start, grouped by the local day they start on.
    pub upcoming: Vec<(Date, Vec<usize>)>,
    pub undated: Vec<usize>,
    /// Most recent first.
    pub past: Vec<usize>,
}

impl Model {
    pub fn build(results: Arc<Vec<Bookmark>>, now: &Zoned) -> Model {
        let tz = now.time_zone();
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
            now: now.timestamp(),
            today: now.date(),
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
        self.today
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
                Phase::Upcoming => {
                    let day = span.start.local.date();
                    match self.upcoming.last_mut() {
                        Some((last, events)) if *last == day => events.push(at),
                        _ => self.upcoming.push((day, vec![at])),
                    }
                }
            }
        }
        self.past.reverse();
        // An event that began on an earlier day joins each later day's list where its
        // start falls, ahead of that day's all-day events: put those back in front.
        let events = &self.events;
        for list in self.by_day.values_mut() {
            list.sort_by_key(|&at| events[at].span.map(|s| (!s.all_day, s.start)));
        }
    }
}

#[cfg(test)]
mod tests {
    use jiff::{
        civil::{DateTime, date},
        tz::TimeZone,
    };
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
            summary_origin: knoa_core::Origin::Ai,
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
        let now = now.to_zoned(tz).expect("a time Warsaw's clocks show");
        Model::build(Arc::new(events), &now)
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

    #[test]
    fn an_event_across_the_fall_back_hour_ends_at_its_instant() {
        // Warsaw's clocks go back at 03:00 on 25 October 2026: 02:45 in summer time to
        // 02:15 in winter time is half an hour, though the end's wall time is earlier.
        let model = build(
            vec![event(
                1,
                "2026-10-25T02:45+02:00",
                Some("2026-10-25T02:15+01:00"),
            )],
            date(2026, 10, 25).at(4, 0, 0, 0),
        );
        assert_eq!(model.past, [0]);
        let span = model.events[0].span.expect("a placed event");
        assert!(span.end.is_some(), "the end is after the start as instants");
    }

    #[test]
    fn a_day_lists_all_day_events_before_one_running_on_from_an_earlier_day() {
        let model = build(
            vec![
                event(1, "2026-12-04T10:00", Some("2026-12-06T18:00")),
                event(2, "2026-12-05", None),
                event(3, "2026-12-05T09:00", None),
            ],
            date(2026, 12, 1).at(9, 0, 0, 0),
        );
        let ids: Vec<i64> = model
            .on(date(2026, 12, 5))
            .iter()
            .map(|&i| model.events[i].id.0)
            .collect();
        assert_eq!(ids, [2, 1, 3]);
    }
}
