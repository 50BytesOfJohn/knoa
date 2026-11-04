//! What the library shows: a place, free text and typed filters, parsed from the search bar.

use serde_json::Value;

use crate::{FieldType, FolderId, Kind, field::parse_minutes};

/// The sidebar selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Place {
    #[default]
    Everything,
    UpNext,
    /// Bookmarks in no folder.
    Unsorted,
    /// Bookmarks the AI hasn't organized yet.
    NotOrganized,
    Folder(FolderId),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Query {
    pub place: Place,
    /// Free text, matched with full-text search.
    pub text: String,
    pub kind: Option<Kind>,
    pub tags: Vec<String>,
    pub sites: Vec<String>,
    pub fields: Vec<FieldFilter>,
}

/// A comparison on a typed field: `runtime>2h`, `year:1999`, `cast:keanu`.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldFilter {
    pub name: String,
    pub op: Comparison,
    pub value: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    /// Equal for numbers, "contains" for text and lists.
    Matches,
    Greater,
    Less,
}

impl Query {
    /// Parses search bar input. Tokens with a known prefix become filters, the rest is text:
    /// `kind:movie`, `site:github.com`, `#noir`, `runtime>2h`, `year:1999`.
    pub fn parse(input: &str, place: Place) -> Query {
        let mut query = Query {
            place,
            ..Query::default()
        };
        let mut words = Vec::new();
        for token in input.split_whitespace() {
            if !query.apply_filter(token) {
                words.push(token);
            }
        }
        query.text = words.join(" ");
        query
    }

    pub fn is_filtered(&self) -> bool {
        !self.text.is_empty()
            || self.kind.is_some()
            || !self.tags.is_empty()
            || !self.sites.is_empty()
            || !self.fields.is_empty()
    }

    fn apply_filter(&mut self, token: &str) -> bool {
        if let Some(tag) = token.strip_prefix('#').filter(|t| !t.is_empty()) {
            self.tags.push(tag.to_lowercase());
            return true;
        }
        let Some((name, op, value)) = split_comparison(token) else {
            return false;
        };
        match (name.as_str(), op) {
            ("kind", Comparison::Matches) => match Kind::parse(value) {
                Some(kind) => self.kind = Some(kind),
                None => return false,
            },
            ("site", Comparison::Matches) => self.sites.push(value.to_lowercase()),
            ("tag", Comparison::Matches) => self.tags.push(value.to_lowercase()),
            _ => match field_type(&name) {
                Some(ty) => match filter_value(ty, value) {
                    Some(value) => self.fields.push(FieldFilter { name, op, value }),
                    None => return false,
                },
                None => return false,
            },
        }
        true
    }
}

fn split_comparison(token: &str) -> Option<(String, Comparison, &str)> {
    let index = token.find([':', '>', '<'])?;
    let (name, rest) = token.split_at(index);
    let op = match rest.as_bytes()[0] {
        b'>' => Comparison::Greater,
        b'<' => Comparison::Less,
        _ => Comparison::Matches,
    };
    let value = &rest[1..];
    (!name.is_empty() && !value.is_empty()).then(|| (name.to_lowercase(), op, value))
}

/// Field names are shared across kinds with the same meaning (`duration`, `year`), so the
/// first kind that defines a name decides its type.
fn field_type(name: &str) -> Option<FieldType> {
    Kind::ALL
        .into_iter()
        .find_map(|kind| kind.field(name))
        .map(|def| def.ty)
}

fn filter_value(ty: FieldType, text: &str) -> Option<Value> {
    match ty {
        FieldType::Duration => parse_minutes(text).map(Value::from),
        FieldType::Integer | FieldType::Number => text.parse::<f64>().ok().map(Value::from),
        FieldType::Text | FieldType::List | FieldType::Date => {
            Some(Value::from(text.replace('_', " ")))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn search_text_splits_into_filters_and_words() {
        let query = Query::parse(
            "noir kind:movies runtime>2h #classic site:imdb.com night",
            Place::Everything,
        );
        assert_eq!(query.text, "noir night");
        assert_eq!(query.kind, Some(Kind::Movie));
        assert_eq!(query.tags, ["classic"]);
        assert_eq!(query.sites, ["imdb.com"]);
        assert_eq!(
            query.fields,
            [FieldFilter {
                name: "runtime".into(),
                op: Comparison::Greater,
                value: json!(120)
            }]
        );
    }

    #[test]
    fn unknown_prefixes_stay_text() {
        let query = Query::parse("http://example.com foo:bar", Place::Everything);
        assert_eq!(query.text, "http://example.com foo:bar");
    }
}
