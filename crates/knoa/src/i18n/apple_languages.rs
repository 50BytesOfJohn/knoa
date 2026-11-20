//! The app's own `AppleLanguages` default, which macOS reads at launch to pick the
//! language of system panels (save panels, alerts) and of `NSLocale.preferredLanguages`.

#![expect(
    unsafe_code,
    reason = "NSUserDefaults takes any object, so objc2 can't check the value's type"
)]

use knoa_core::Language;
use objc2::runtime::AnyObject;
use objc2_foundation::{NSArray, NSString, NSUserDefaults};

pub fn set(language: Language) {
    let languages = NSArray::from_retained_slice(&[NSString::from_str(language.code())]);
    let value: &AnyObject = &languages;
    let defaults = NSUserDefaults::standardUserDefaults();
    // SAFETY: `AppleLanguages` holds an array of language codes, and `value` is one.
    unsafe { defaults.setObject_forKey(Some(value), &NSString::from_str("AppleLanguages")) };
}
