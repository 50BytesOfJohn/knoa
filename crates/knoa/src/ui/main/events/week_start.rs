//! The first day of the week in the user's region settings, for the month grid.

use jiff::civil::Weekday;
use objc2_foundation::NSCalendar;

/// The weekday macOS starts weeks on (System Settings > General > Language & Region), or
/// Monday if it names none.
pub fn first_weekday() -> Weekday {
    // NSCalendar counts weekdays from 1, Sunday, to 7, Saturday.
    let first = NSCalendar::autoupdatingCurrentCalendar().firstWeekday();
    i8::try_from(first)
        .ok()
        .and_then(|n| Weekday::from_sunday_one_offset(n).ok())
        .unwrap_or(Weekday::Monday)
}
