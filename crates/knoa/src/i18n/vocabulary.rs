//! Names from the core's data, like kinds, fields and inks, in the current language. Their
//! message IDs are built at run time, so a test checks that each one exists.

use knoa_core::{FieldDef, Ink, Kind};

use super::LOADER;

pub fn kind_label(kind: Kind) -> String {
    LOADER.get(&format!("kind-{}", kind.slug()))
}

/// The name of a kind's collection, as on its chip: "Movies", "Code".
pub fn kind_plural(kind: Kind) -> String {
    LOADER.get(&format!("kind-{}-plural", kind.slug()))
}

pub fn field_label(def: &FieldDef) -> String {
    LOADER.get(&field_id(&def.name))
}

/// A color's name, as in the color pickers' tooltips: "Ochre".
pub fn ink_label(ink: Ink) -> String {
    LOADER.get(&ink_id(ink))
}

/// Search input with `kind:` filters in the current language ("kind:filmy") turned into
/// the slugs the query parser reads ("kind:movie").
pub fn with_kind_slugs(input: &str) -> String {
    let tokens: Vec<String> = input
        .split_whitespace()
        .map(|token| match token.split_once(':') {
            Some((name, value)) if name.eq_ignore_ascii_case("kind") => {
                kind_named(value).map_or_else(|| token.to_owned(), |k| format!("kind:{}", k.slug()))
            }
            _ => token.to_owned(),
        })
        .collect();
    tokens.join(" ")
}

fn kind_named(name: &str) -> Option<Kind> {
    let name = name.to_lowercase();
    Kind::ALL.into_iter().find(|&kind| {
        kind_label(kind).to_lowercase() == name || kind_plural(kind).to_lowercase() == name
    })
}

fn field_id(name: &str) -> String {
    format!("field-{}", name.replace('_', "-"))
}

fn ink_id(ink: Ink) -> String {
    format!("ink-{}", ink.label().to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_field_and_ink_has_a_message() {
        for kind in Kind::ALL {
            let slug = kind.slug();
            assert!(
                LOADER.has(&format!("kind-{slug}")),
                "kind-{slug} is missing"
            );
            assert!(
                LOADER.has(&format!("kind-{slug}-plural")),
                "kind-{slug}-plural is missing"
            );
            for def in &kind.definition().fields {
                let id = field_id(&def.name);
                assert!(LOADER.has(&id), "{id} is missing");
            }
        }
        for ink in Ink::ALL {
            let id = ink_id(ink);
            assert!(LOADER.has(&id), "{id} is missing");
        }
    }
}
