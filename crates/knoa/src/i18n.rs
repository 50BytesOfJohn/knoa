//! The app's language: Fluent messages through [`t!`], and switching languages while the
//! app runs.

mod apple_languages;
mod vocabulary;

use std::sync::LazyLock;

use gpui::{App, Context, Global, Subscription, Window};
use i18n_embed::{
    LanguageLoader as _, fluent::FluentLanguageLoader, unic_langid::LanguageIdentifier,
};
use knoa_core::Language;
use objc2_foundation::NSLocale;
use rust_embed::RustEmbed;

pub use vocabulary::{
    field_label, ink_label, kind_label, kind_plural, with_kind_names, with_kind_slugs,
};

use crate::state::settings::SettingsFile;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

/// Speaks English until [`init`] picks the user's language.
pub static LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader = FluentLanguageLoader::new("knoa", identifier(Language::En));
    load(&loader, Language::En);
    loader
});

/// A message in the current language: `t!("menu-hide-others")`, or with arguments,
/// `t!("onboarding-browser-count", count = 3)`. The ID is checked against the English file
/// at compile time.
macro_rules! t {
    ($($args:tt)*) => {
        i18n_embed_fl::fl!(crate::i18n::LOADER, $($args)*)
    };
}
pub(crate) use t;

/// The language messages and dates are in now.
pub fn language() -> Language {
    Language::parse(LOADER.current_language().language.as_str()).unwrap_or_default()
}

/// The Mac's first preferred language when Knoa speaks it, else English.
pub fn system_language() -> Language {
    NSLocale::preferredLanguages()
        .firstObject()
        .and_then(|code| Language::parse(&code.to_string()))
        .unwrap_or_default()
}

/// Set again on every language switch, which tells its observers. See [`on_change`].
struct LanguageChanged;

impl Global for LanguageChanged {}

/// Calls `translate` whenever the language changes, for text a view keeps rather than
/// draws each frame, like an input's placeholder.
pub fn on_change<T: 'static>(
    window: &Window,
    cx: &mut Context<'_, T>,
    translate: impl FnMut(&mut T, &mut Window, &mut Context<'_, T>) + 'static,
) -> Subscription {
    cx.observe_global_in::<LanguageChanged>(window, translate)
}

/// Sets the language at launch, before any window or menu exists.
pub fn init(language: Language) {
    load(&LOADER, language);
    gpui_component::set_locale(language.code());
    apple_languages::set(language);
}

/// Switches the whole app to `language` and keeps it in settings. Every window draws
/// again, so views show it at once.
pub fn set_language(language: Language, cx: &mut App) {
    if SettingsFile::get(cx).language != language {
        SettingsFile::update(cx, |s| s.language = language);
    }
    if self::language() == language {
        return;
    }
    tracing::info!(language = language.code(), "switching language");
    init(language);
    crate::menus::set(cx);
    cx.set_global(LanguageChanged);
    cx.refresh_windows();
}

fn load(loader: &FluentLanguageLoader, language: Language) {
    if let Err(err) = loader.load_languages(&Localizations, &[identifier(language)]) {
        tracing::error!("can't load the {} messages: {err}", language.english_name());
    }
    // GPUI would draw the Unicode isolation marks Fluent puts around arguments.
    loader.set_use_isolating(false);
}

fn identifier(language: Language) -> LanguageIdentifier {
    language
        .code()
        .parse()
        .expect("language codes are valid language identifiers")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use fluent_syntax::{ast, parser};

    use super::*;

    /// Every message and attribute in a file, as `id` and `id.attribute`.
    fn keys(language: Language) -> BTreeSet<String> {
        let path = format!("{}/knoa.ftl", language.code());
        let file = Localizations::get(&path).unwrap_or_else(|| panic!("{path} is missing"));
        let text = String::from_utf8(file.data.into_owned()).expect("FTL files are UTF-8");
        let resource = parser::parse(text.as_str())
            .unwrap_or_else(|(_, errors)| panic!("{path} doesn't parse: {errors:?}"));
        let mut keys = BTreeSet::new();
        for entry in resource.body {
            if let ast::Entry::Message(message) = entry {
                for attribute in &message.attributes {
                    keys.insert(format!("{}.{}", message.id.name, attribute.id.name));
                }
                keys.insert(message.id.name.to_owned());
            }
        }
        keys
    }

    #[test]
    fn every_language_has_exactly_the_english_messages() {
        let english = keys(Language::En);
        for language in Language::ALL {
            let translated = keys(language);
            let missing: Vec<_> = english.difference(&translated).collect();
            let extra: Vec<_> = translated.difference(&english).collect();
            assert!(
                missing.is_empty() && extra.is_empty(),
                "{}: missing {missing:?}, not in English {extra:?}",
                language.code()
            );
        }
    }
}
