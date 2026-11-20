//! Dates and times as the app's language writes them, through macOS's own formatters.
//!
//! Each format is a Unicode skeleton, like `"yMMMd"`: which parts to show, not their
//! order or punctuation, which the language and the Mac's region decide.

use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use jiff::{Timestamp, civil, tz::TimeZone};
use knoa_core::Language;
use objc2::rc::Retained;
use objc2_foundation::{
    NSDate, NSDateFormatter, NSDateIntervalFormatter, NSFormattingContext, NSLocale, NSString,
    NSTimeZone,
};

use crate::i18n;

/// "19:00", or "7:00 PM" where the region uses a 12-hour clock.
pub const TIME: &str = "jm";
/// "3 Nov 2026".
pub const DATE: &str = "yMMMd";
/// "3 November".
pub const DAY_MONTH_LONG: &str = "MMMMd";
/// "3 November 2026".
pub const DATE_LONG: &str = "yMMMMd";
/// "5 Dec".
pub const DAY_MONTH: &str = "dMMM";
/// "30 Nov, 20:00".
pub const DAY_MONTH_TIME: &str = "dMMMjm";
/// "Sat 12 Dec".
pub const WEEKDAY_DATE: &str = "EEEdMMM";
/// "Sat 12 Dec 2027".
pub const WEEKDAY_DATE_YEAR: &str = "yEEEdMMM";
/// "Sat 12 Dec 2027, 19:00".
pub const WEEKDAY_DATE_YEAR_TIME: &str = "yEEEdMMMjm";
/// "December", standing alone, as a heading.
pub const MONTH: &str = "LLLL";
/// "Dec", standing alone.
pub const MONTH_SHORT: &str = "LLL";
/// "December 2026".
pub const MONTH_YEAR: &str = "yMMMM";
/// "Dec 2027".
pub const MONTH_YEAR_SHORT: &str = "yMMM";
/// "Saturday", standing alone.
pub const WEEKDAY: &str = "cccc";
/// "Sat", standing alone.
pub const WEEKDAY_SHORT: &str = "ccc";

/// An instant, in this Mac's time zone.
pub fn instant(at: Timestamp, skeleton: &'static str) -> String {
    let formatter = formatter(skeleton, Zone::Local);
    formatter.stringFromDate(&ns_date(at)).to_string()
}

/// A wall time with no offset, as written.
pub fn floating(at: civil::DateTime, skeleton: &'static str) -> String {
    let formatter = formatter(skeleton, Zone::Utc);
    formatter.stringFromDate(&floating_date(at)).to_string()
}

pub fn day(date: civil::Date, skeleton: &'static str) -> String {
    floating(date.to_datetime(civil::Time::midnight()), skeleton)
}

/// From one wall time to another, sharing what they have in common: "5 – 7 Dec",
/// "19:00 – 23:00".
pub fn floating_range(
    start: civil::DateTime,
    end: civil::DateTime,
    skeleton: &'static str,
) -> String {
    let formatter = interval_formatter(skeleton);
    formatter
        .stringFromDate_toDate(&floating_date(start), &floating_date(end))
        .to_string()
}

/// Formatters are slow to make, so each language, skeleton and zone gets one, kept.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Zone {
    Local,
    /// Floating times go in and come out as UTC, so no zone shifts them.
    Utc,
}

type Key = (Language, &'static str, Zone);

static FORMATTERS: LazyLock<Mutex<HashMap<Key, Retained<NSDateFormatter>>>> =
    LazyLock::new(Mutex::default);
static INTERVAL_FORMATTERS: LazyLock<Mutex<HashMap<Key, Retained<NSDateIntervalFormatter>>>> =
    LazyLock::new(Mutex::default);

fn formatter(skeleton: &'static str, zone: Zone) -> Retained<NSDateFormatter> {
    let language = i18n::language();
    let mut formatters = FORMATTERS.lock().expect("lock poisoned");
    let formatter = formatters
        .entry((language, skeleton, zone))
        .or_insert_with(|| {
            let formatter = NSDateFormatter::new();
            formatter.setLocale(Some(&locale(language)));
            formatter.setTimeZone(Some(&time_zone(zone)));
            // Every date shown either stands alone or starts with a number, so a month or
            // weekday that comes first is capitalized as the language does there.
            formatter.setFormattingContext(NSFormattingContext::BeginningOfSentence);
            formatter.setLocalizedDateFormatFromTemplate(&NSString::from_str(skeleton));
            formatter
        });
    formatter.clone()
}

fn interval_formatter(skeleton: &'static str) -> Retained<NSDateIntervalFormatter> {
    let language = i18n::language();
    let mut formatters = INTERVAL_FORMATTERS.lock().expect("lock poisoned");
    let formatter = formatters
        .entry((language, skeleton, Zone::Utc))
        .or_insert_with(|| {
            let formatter = NSDateIntervalFormatter::new();
            formatter.setLocale(Some(&locale(language)));
            formatter.setTimeZone(Some(&time_zone(Zone::Utc)));
            formatter.setDateTemplate(Some(&NSString::from_str(skeleton)));
            formatter
        });
    formatter.clone()
}

/// The app's language with the Mac's region, so a Pole in Germany gets Polish words
/// with German conventions, as macOS does for apps.
#[expect(deprecated, reason = "its replacement, regionCode, needs macOS 13")]
fn locale(language: Language) -> Retained<NSLocale> {
    let identifier = match NSLocale::currentLocale().countryCode() {
        Some(region) => format!("{}_{region}", language.code()),
        None => language.code().to_owned(),
    };
    NSLocale::localeWithLocaleIdentifier(&NSString::from_str(&identifier))
}

fn time_zone(zone: Zone) -> Retained<NSTimeZone> {
    match zone {
        // Follows the Mac when its time zone changes.
        Zone::Local => NSTimeZone::localTimeZone(),
        Zone::Utc => NSTimeZone::timeZoneForSecondsFromGMT(0),
    }
}

fn ns_date(at: Timestamp) -> Retained<NSDate> {
    NSDate::dateWithTimeIntervalSince1970(at.as_millisecond() as f64 / 1000.)
}

fn floating_date(at: civil::DateTime) -> Retained<NSDate> {
    let utc = at
        .to_zoned(TimeZone::UTC)
        .map_or(Timestamp::UNIX_EPOCH, |zoned| zoned.timestamp());
    ns_date(utc)
}
