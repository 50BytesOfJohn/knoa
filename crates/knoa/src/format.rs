//! Dates, numbers and lists as the app's language writes them, with the Mac's region's
//! conventions, through macOS's own formatters. Date and number formatters are slow to
//! make and used on every frame, so those modules keep the ones they made, per language.

mod dates;
mod list;
mod numbers;

use knoa_core::Language;
use objc2::rc::Retained;
use objc2_foundation::{NSLocale, NSString};

pub use dates::{
    DATE, DATE_LONG, DAY_MONTH, DAY_MONTH_LONG, DAY_MONTH_TIME, MONTH, MONTH_SHORT, MONTH_YEAR,
    MONTH_YEAR_SHORT, TIME, WEEKDAY, WEEKDAY_DATE, WEEKDAY_DATE_YEAR, WEEKDAY_DATE_YEAR_TIME,
    WEEKDAY_SHORT, day, floating, floating_range, instant,
};
pub use list::list;
pub use numbers::{compact, number, usd};

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
