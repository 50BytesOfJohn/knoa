//! `AppleLanguages`, the preferred languages: the app's own default, which macOS reads at
//! launch to pick the language of system panels (save panels, alerts) and of
//! `NSLocale.preferredLanguages`, and the Mac's, set in System Settings.

#![expect(
    unsafe_code,
    reason = "NSUserDefaults takes any object, so objc2 can't check the value's type"
)]

use knoa_core::Language;
use objc2::runtime::AnyObject;
use objc2_foundation::{NSArray, NSString, NSUserDefaults, ns_string};

pub fn set(language: Language) {
    let languages = NSArray::from_retained_slice(&[NSString::from_str(language.code())]);
    let value: &AnyObject = &languages;
    let defaults = NSUserDefaults::standardUserDefaults();
    // SAFETY: `AppleLanguages` holds an array of language codes, and `value` is one.
    unsafe { defaults.setObject_forKey(Some(value), &NSString::from_str("AppleLanguages")) };
}

/// The Mac's preferred language tags, most preferred first. Read from the global domain
/// because the app's own default, which [`set`] writes and a reinstall keeps, shadows
/// them in `NSLocale.preferredLanguages`.
pub fn system() -> Vec<String> {
    let global = NSUserDefaults::standardUserDefaults()
        .persistentDomainForName(ns_string!("NSGlobalDomain"));
    let languages = global
        .and_then(|global| global.objectForKey(ns_string!("AppleLanguages")))
        .and_then(|languages| languages.downcast::<NSArray>().ok());
    let Some(languages) = languages else {
        return Vec::new();
    };
    languages
        .iter()
        .filter_map(|tag| tag.downcast::<NSString>().ok())
        .map(|tag| tag.to_string())
        .collect()
}
