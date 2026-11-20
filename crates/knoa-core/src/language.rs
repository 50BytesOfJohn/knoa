//! The user's language: for every client's UI and for what the AI writes.

use serde::{Deserialize, Deserializer, Serialize};

/// On the wire and in settings as the bare ISO 639-1 code, like `"pl"`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    En,
    Pl,
    De,
    Es,
    Fr,
}

impl Language {
    pub const ALL: [Language; 5] = [Self::En, Self::Pl, Self::De, Self::Es, Self::Fr];

    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Pl => "pl",
            Self::De => "de",
            Self::Es => "es",
            Self::Fr => "fr",
        }
    }

    /// A code like `"pl"`, or a locale tag like `"pl-PL"` or `"de_AT"`, in any case.
    /// `None` for a language Knoa doesn't speak.
    pub fn parse(code: &str) -> Option<Self> {
        let primary = code.split(['-', '_']).next()?;
        Self::ALL
            .into_iter()
            .find(|language| language.code().eq_ignore_ascii_case(primary))
    }

    /// The first of the user's preferred locale tags, most preferred first, that Knoa
    /// speaks, else English.
    pub fn first_spoken(tags: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        tags.into_iter()
            .find_map(|tag| Self::parse(tag.as_ref()))
            .unwrap_or_default()
    }

    /// The name in English, for prompts: models follow "Write in Polish" best.
    pub fn english_name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Pl => "Polish",
            Self::De => "German",
            Self::Es => "Spanish",
            Self::Fr => "French",
        }
    }

    /// The name in the language itself, for pickers, so anyone can find their own.
    pub fn native_name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Pl => "Polski",
            Self::De => "Deutsch",
            Self::Es => "Español",
            Self::Fr => "Français",
        }
    }
}

/// For `#[serde(deserialize_with)]` on a `Language` or `Option<Language>` field: a code
/// this build doesn't know reads as the default, `En` or `None`, so a newer server or a
/// newer Knoa adding a language doesn't make the whole value unreadable.
///
/// # Errors
///
/// Only when the input isn't valid for the format at all.
pub fn unknown_language_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(T::deserialize(value).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_takes_codes_and_locale_tags_and_rejects_the_rest() {
        assert_eq!(Language::parse("pl"), Some(Language::Pl));
        assert_eq!(Language::parse("de-AT"), Some(Language::De));
        assert_eq!(Language::parse("es_MX"), Some(Language::Es));
        assert_eq!(Language::parse("FR"), Some(Language::Fr));
        assert_eq!(Language::parse("pt"), None);
        assert_eq!(Language::parse("pt-BR"), None);
        assert_eq!(Language::parse(""), None);
    }

    #[test]
    fn first_spoken_skips_languages_knoa_does_not_speak() {
        assert_eq!(
            Language::first_spoken(["it-IT", "de-DE", "pl-PL"]),
            Language::De
        );
        assert_eq!(Language::first_spoken(["it-IT", "pt-BR"]), Language::En);
    }

    #[test]
    fn unknown_language_as_default_keeps_known_codes_and_defaults_unknown_ones() {
        #[derive(Deserialize)]
        struct Holder {
            #[serde(deserialize_with = "unknown_language_as_default")]
            language: Option<Language>,
        }
        let read = |json: &str| {
            serde_json::from_str::<Holder>(json)
                .expect("an unknown language still reads")
                .language
        };

        assert_eq!(read(r#"{"language":"pl"}"#), Some(Language::Pl));
        assert_eq!(read(r#"{"language":"pt"}"#), None);
    }
}
