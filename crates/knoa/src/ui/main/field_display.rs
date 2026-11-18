//! Field values as the user reads them: `knoa_core`'s display, but a time with an offset
//! shows in this Mac's time zone, where the Events page places it.

use jiff::tz::TimeZone;
use knoa_core::{Bookmark, Date, EventTime, FieldDef, FieldType, TimeOfDay};
use serde_json::Value;

use super::events::local_time;

pub(super) fn display(def: &FieldDef, value: &Value) -> String {
    match local_event_time(def, value) {
        Some(time) => time.display(),
        None => def.display(value),
    }
}

pub(super) fn key_facts(bookmark: &Bookmark) -> Vec<String> {
    bookmark.key_facts(display)
}

/// A datetime value with an offset as the same instant in local wall time. `None` for
/// anything else, which shows as written.
fn local_event_time(def: &FieldDef, value: &Value) -> Option<EventTime> {
    if def.ty != FieldType::DateTime {
        return None;
    }
    let time = EventTime::parse(value.as_str()?)?;
    time.time?.offset_minutes?;
    let local = local_time(&time, &TimeZone::system())?;
    let date = Date::new(
        u16::try_from(local.year()).ok()?,
        u8::try_from(local.month()).ok()?,
        u8::try_from(local.day()).ok()?,
    )?;
    Some(EventTime {
        date,
        time: Some(TimeOfDay {
            hour: u8::try_from(local.hour()).ok()?,
            minute: u8::try_from(local.minute()).ok()?,
            offset_minutes: None,
        }),
    })
}
