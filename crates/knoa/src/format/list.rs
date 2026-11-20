//! Lists of names, joined as the language joins them.

#![expect(
    unsafe_code,
    reason = "NSListFormatter takes an array of any objects, so objc2 can't check them"
)]

use objc2::{rc::Retained, runtime::AnyObject};
use objc2_foundation::{NSArray, NSListFormatter, NSString};

use super::locale;
use crate::i18n;

/// Names joined as the language lists them: "Chrome and Brave", "Chrome, Brave and Edge".
pub fn list(items: &[&str]) -> String {
    let formatter = NSListFormatter::new();
    formatter.setLocale(Some(&locale(i18n::language())));
    let strings: Vec<Retained<NSString>> =
        items.iter().map(|item| NSString::from_str(item)).collect();
    let objects: Vec<&AnyObject> = strings
        .iter()
        .map(|string| -> &AnyObject { string })
        .collect();
    let array = NSArray::from_slice(&objects);
    // SAFETY: the formatter reads each item's description, and NSStrings have one.
    let joined = unsafe { formatter.stringFromItems(&array) };
    joined.map(|joined| joined.to_string()).unwrap_or_default()
}
