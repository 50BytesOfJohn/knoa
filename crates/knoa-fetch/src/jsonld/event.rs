//! An `Event` item's fields: its times, where it happens, who runs it, how to attend.

use knoa_core::{Fields, Kind};
use serde_json::Value;

use super::{lookup, names, type_of};

/// schema.org's Event and its subtypes that are something a person attends or watches.
/// Deliveries, publications and sales are Events to schema.org but not to anyone else.
pub(super) const TYPES: &[&str] = &[
    "Event",
    "BusinessEvent",
    "ChildrensEvent",
    "ComedyEvent",
    "ConferenceEvent",
    "CourseInstance",
    "DanceEvent",
    "EducationEvent",
    "EventSeries",
    "ExhibitionEvent",
    "Festival",
    "FoodEvent",
    "Hackathon",
    "LiteraryEvent",
    "MusicEvent",
    "PerformingArtsEvent",
    "ScreeningEvent",
    "SocialEvent",
    "SportsEvent",
    "TheaterEvent",
    "VisualArtsEvent",
];

pub(super) fn fields(item: &Value) -> Fields {
    let mut fields = Fields::new();
    let mut set = |name: &str, raw: Option<Value>| {
        let def = Kind::Event
            .field(name)
            .expect("event fields are in kinds.json");
        if let Some(value) = raw.and_then(|raw| def.coerce(&raw)) {
            fields.insert(name.to_owned(), value);
        }
    };
    set("starts", item.get("startDate").cloned());
    set("ends", item.get("endDate").cloned());
    let location = Location::of(item.get("location"));
    set("venue", location.venue());
    if let Some(place) = location.place {
        set("city", lookup(place, "address.addressLocality").cloned());
        set(
            "country",
            lookup(place, "address.addressCountry").map(names),
        );
    }
    set("organizer", item.get("organizer").map(joined_names));
    let mode = item.get("eventAttendanceMode").and_then(Value::as_str);
    set("attendance", attendance(mode, &location).map(Value::from));
    fields
}

/// Where an event happens: `location` is a place, an online location, a bare name, or a
/// list of those.
#[derive(Default)]
struct Location<'a> {
    place: Option<&'a Value>,
    name: Option<&'a str>,
    online: bool,
}

impl<'a> Location<'a> {
    fn of(value: Option<&'a Value>) -> Location<'a> {
        let mut location = Location::default();
        let items = match value {
            Some(Value::Array(items)) => items.as_slice(),
            Some(item) => std::slice::from_ref(item),
            None => &[],
        };
        for item in items {
            match item {
                Value::String(name) => location.name = location.name.or(Some(name)),
                Value::Object(_) if type_of(item).as_deref() == Some("VirtualLocation") => {
                    location.online = true;
                }
                Value::Object(_) => location.place = location.place.or(Some(item)),
                _ => {}
            }
        }
        location
    }

    fn venue(&self) -> Option<Value> {
        let named = self.place.and_then(|place| place.get("name")).cloned();
        let named = named.or_else(|| self.name.map(Value::from));
        named.or_else(|| self.online.then(|| Value::from("Online")))
    }
}

/// `eventAttendanceMode` may be a full URL: `https://schema.org/OfflineEventAttendanceMode`.
fn attendance(mode: Option<&str>, location: &Location<'_>) -> Option<&'static str> {
    let mode = mode.map(|m| m.rsplit('/').next().unwrap_or(m));
    match mode {
        Some("OfflineEventAttendanceMode") => Some("in person"),
        Some("OnlineEventAttendanceMode") => Some("online"),
        Some("MixedEventAttendanceMode") => Some("hybrid"),
        _ => match (location.place.is_some(), location.online) {
            (true, true) => Some("hybrid"),
            (false, true) => Some("online"),
            _ => None,
        },
    }
}

/// Organizers come as one or several people or organizations; the field is one line.
fn joined_names(value: &Value) -> Value {
    match names(value) {
        Value::Array(items) => {
            let names: Vec<&str> = items.iter().filter_map(Value::as_str).collect();
            Value::from(names.join(", "))
        }
        other => other,
    }
}
