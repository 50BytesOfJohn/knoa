//! The fixed set of bookmark kinds. Their fields are data, loaded from `data/kinds.json`.

use std::{collections::HashMap, sync::LazyLock};

use serde::Deserialize;

use crate::field::FieldDef;

/// What a bookmark is. The order is the order of the kind chips.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Article,
    Video,
    Movie,
    Recipe,
    Product,
    Repository,
    Book,
    Image,
    App,
    Podcast,
    Place,
    Other,
}

/// A kind's definition: what it means and which typed fields it has.
#[derive(Debug, Deserialize)]
pub struct KindDef {
    pub label: String,
    pub plural: String,
    pub description: String,
    /// Fields shown next to the site on a card, in order.
    pub key_facts: Vec<String>,
    pub fields: Vec<FieldDef>,
}

static DEFINITIONS: LazyLock<HashMap<String, KindDef>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../data/kinds.json"))
        .expect("data/kinds.json is checked by the kinds_file_matches_enum test")
});

impl Kind {
    pub const ALL: [Kind; 12] = [
        Kind::Article,
        Kind::Video,
        Kind::Movie,
        Kind::Recipe,
        Kind::Product,
        Kind::Repository,
        Kind::Book,
        Kind::Image,
        Kind::App,
        Kind::Podcast,
        Kind::Place,
        Kind::Other,
    ];

    /// The stable identifier used in the database, prompts and `kind:` filters.
    pub fn slug(self) -> &'static str {
        match self {
            Kind::Article => "article",
            Kind::Video => "video",
            Kind::Movie => "movie",
            Kind::Recipe => "recipe",
            Kind::Product => "product",
            Kind::Repository => "repository",
            Kind::Book => "book",
            Kind::Image => "image",
            Kind::App => "app",
            Kind::Podcast => "podcast",
            Kind::Place => "place",
            Kind::Other => "other",
        }
    }

    /// Parses a slug, also accepting labels and plurals typed in the search bar ("movies").
    pub fn parse(text: &str) -> Option<Kind> {
        let text = text.trim().to_lowercase();
        Kind::ALL.into_iter().find(|kind| {
            let def = kind.definition();
            kind.slug() == text
                || def.label.to_lowercase() == text
                || def.plural.to_lowercase() == text
        })
    }

    pub fn definition(self) -> &'static KindDef {
        &DEFINITIONS[self.slug()]
    }

    pub fn label(self) -> &'static str {
        &self.definition().label
    }

    pub fn plural(self) -> &'static str {
        &self.definition().plural
    }

    pub fn field(self, name: &str) -> Option<&'static FieldDef> {
        self.definition().fields.iter().find(|f| f.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_file_matches_enum() {
        assert_eq!(DEFINITIONS.len(), Kind::ALL.len());
        for kind in Kind::ALL {
            let def = kind.definition();
            for fact in &def.key_facts {
                assert!(
                    kind.field(fact).is_some(),
                    "{kind:?} key fact {fact} is not a field"
                );
            }
        }
    }
}
