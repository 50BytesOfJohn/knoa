//! The app's language: Fluent messages through [`t!`], and switching languages while the
//! app runs.

mod apple_languages;
mod vocabulary;

use std::sync::LazyLock;

use fluent_bundle::FluentValue;
use gpui::{App, Context, Global, Subscription, Window};
use i18n_embed::{
    LanguageLoader as _, fluent::FluentLanguageLoader, unic_langid::LanguageIdentifier,
};
use knoa_core::Language;
use objc2_foundation::NSLocale;
use rust_embed::RustEmbed;

pub use vocabulary::{
    field_label, glyph_label, ink_label, kind_label, kind_plural, with_kind_names, with_kind_slugs,
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
    loader.with_bundles_mut(|bundle| bundle.set_formatter(Some(number_argument)));
}

/// Numbers in messages, like `{ $count }`, as the language and the Mac's region write
/// them: Fluent alone writes 12345 in every language.
fn number_argument<M>(value: &FluentValue<'_>, _: &M) -> Option<String> {
    match value {
        FluentValue::Number(number) => Some(crate::format::number(number.value)),
        _ => None,
    }
}

fn identifier(language: Language) -> LanguageIdentifier {
    language
        .code()
        .parse()
        .expect("language codes are valid language identifiers")
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use fluent_syntax::{ast, parser};

    use super::*;

    #[test]
    fn every_language_has_exactly_the_english_messages() {
        let english = keys(&resource(Language::En));
        for language in Language::ALL {
            let translated = keys(&resource(language));
            let missing: Vec<_> = english.difference(&translated).collect();
            let extra: Vec<_> = translated.difference(&english).collect();
            assert!(
                missing.is_empty() && extra.is_empty(),
                "{}: missing {missing:?}, not in English {extra:?}",
                language.code()
            );
        }
    }

    /// Fluent prints a variable the caller doesn't pass, or a term the file lacks, as its
    /// name, so a renamed `$count` would only show up as "{$count}" on screen.
    #[test]
    fn translations_use_english_variables_and_existing_terms() {
        let english = references(&resource(Language::En));
        for language in Language::ALL {
            let resource = resource(language);
            let terms = terms(&resource);
            for (id, used) in references(&resource) {
                // A message English lacks fails the test above.
                let Some(allowed) = english.get(&id).map(|e| &e.variables) else {
                    continue;
                };
                let unknown: Vec<_> = used.variables.difference(allowed).collect();
                let missing: Vec<_> = used.terms.difference(&terms).collect();
                assert!(
                    unknown.is_empty() && missing.is_empty(),
                    "{} {id}: variables not in English {unknown:?}, undefined terms {missing:?}",
                    language.code()
                );
            }
        }
    }

    fn resource(language: Language) -> ast::Resource<String> {
        let path = format!("{}/knoa.ftl", language.code());
        let file = Localizations::get(&path).unwrap_or_else(|| panic!("{path} is missing"));
        let text = String::from_utf8(file.data.into_owned()).expect("FTL files are UTF-8");
        parser::parse(text).unwrap_or_else(|(_, errors)| panic!("{path} doesn't parse: {errors:?}"))
    }

    fn messages(resource: &ast::Resource<String>) -> impl Iterator<Item = &ast::Message<String>> {
        resource.body.iter().filter_map(|entry| match entry {
            ast::Entry::Message(message) => Some(message),
            _ => None,
        })
    }

    /// Every message and attribute in a file, as `id` and `id.attribute`.
    fn keys(resource: &ast::Resource<String>) -> BTreeSet<String> {
        let mut keys = BTreeSet::new();
        for message in messages(resource) {
            for attribute in &message.attributes {
                keys.insert(format!("{}.{}", message.id.name, attribute.id.name));
            }
            keys.insert(message.id.name.clone());
        }
        keys
    }

    fn terms(resource: &ast::Resource<String>) -> BTreeSet<String> {
        let terms = resource.body.iter().filter_map(|entry| match entry {
            ast::Entry::Term(term) => Some(term.id.name.clone()),
            _ => None,
        });
        terms.collect()
    }

    /// The variables and terms each message uses anywhere: value, attributes and every
    /// selector variant.
    #[derive(Default)]
    struct References {
        variables: BTreeSet<String>,
        terms: BTreeSet<String>,
    }

    fn references(resource: &ast::Resource<String>) -> BTreeMap<String, References> {
        let mut all = BTreeMap::new();
        for message in messages(resource) {
            let mut found = References::default();
            let attributes = message.attributes.iter().map(|a| &a.value);
            for pattern in message.value.iter().chain(attributes) {
                found.add_pattern(pattern);
            }
            all.insert(message.id.name.clone(), found);
        }
        all
    }

    impl References {
        fn add_pattern(&mut self, pattern: &ast::Pattern<String>) {
            for element in &pattern.elements {
                if let ast::PatternElement::Placeable { expression } = element {
                    self.add_expression(expression);
                }
            }
        }

        fn add_expression(&mut self, expression: &ast::Expression<String>) {
            match expression {
                ast::Expression::Select { selector, variants } => {
                    self.add_inline(selector);
                    for variant in variants {
                        self.add_pattern(&variant.value);
                    }
                }
                ast::Expression::Inline(inline) => self.add_inline(inline),
            }
        }

        fn add_inline(&mut self, inline: &ast::InlineExpression<String>) {
            use ast::InlineExpression::*;
            match inline {
                VariableReference { id } => {
                    self.variables.insert(id.name.clone());
                }
                TermReference { id, arguments, .. } => {
                    self.terms.insert(id.name.clone());
                    if let Some(arguments) = arguments {
                        self.add_arguments(arguments);
                    }
                }
                FunctionReference { arguments, .. } => self.add_arguments(arguments),
                Placeable { expression } => self.add_expression(expression),
                StringLiteral { .. } | NumberLiteral { .. } | MessageReference { .. } => {}
            }
        }

        fn add_arguments(&mut self, arguments: &ast::CallArguments<String>) {
            let named = arguments.named.iter().map(|argument| &argument.value);
            for value in arguments.positional.iter().chain(named) {
                self.add_inline(value);
            }
        }
    }
}
