//! One search word with known syntax, and its typed form: the single source of the search
//! syntax.

use serde_json::Value;

use crate::{FieldType, Kind, field::parse_minutes};

use super::{Comparison, FieldFilter};

/// A filter the search bar can type: `kind:recipe`, `#pasta`, `site:youtube.com`,
/// `runtime>2h`, `director:christopher_nolan`.
#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    Kind(Kind),
    Tag(String),
    Site(String),
    Field(FieldFilter),
}

impl Filter {
    /// One search word with known syntax (`#pasta`, `kind:recipe`, `site:x`, `tag:x`,
    /// `runtime>2h`, `director:nolan`), or `None` if it is plain text.
    pub fn parse(word: &str) -> Option<Filter> {
        if let Some(tag) = word.strip_prefix('#').filter(|t| !t.is_empty()) {
            return Some(Filter::Tag(tag.to_lowercase()));
        }
        let (name, op, value) = split_comparison(word)?;
        match (name.as_str(), op) {
            ("kind", Comparison::Matches) => Kind::parse(value).map(Filter::Kind),
            ("site", Comparison::Matches) => Some(Filter::Site(value.to_lowercase())),
            ("tag", Comparison::Matches) => Some(Filter::Tag(value.to_lowercase())),
            _ => {
                let value = filter_value(field_type(&name)?, value)?;
                Some(Filter::Field(FieldFilter { name, op, value }))
            }
        }
    }

    /// The typed form, which [`Filter::parse`] reads back to the same filter: `kind:recipe`,
    /// `#pasta`, `site:youtube.com`, `director:christopher_nolan`, `runtime>120`.
    pub fn typed(&self) -> String {
        match self {
            Filter::Kind(kind) => format!("kind:{}", kind.slug()),
            Filter::Tag(tag) => format!("#{tag}"),
            Filter::Site(site) => format!("site:{site}"),
            Filter::Field(field) => {
                let op = match field.op {
                    Comparison::Matches => ':',
                    Comparison::Greater => '>',
                    Comparison::Less => '<',
                };
                format!("{}{op}{}", field.name, typed_value(&field.value))
            }
        }
    }
}

fn split_comparison(word: &str) -> Option<(String, Comparison, &str)> {
    let index = word.find([':', '>', '<'])?;
    let (name, rest) = word.split_at(index);
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
        FieldType::Text | FieldType::List | FieldType::Date | FieldType::DateTime => {
            Some(Value::from(text.replace('_', " ")))
        }
    }
}

/// A search word ends at whitespace, so spaces in a value are typed as `_`.
fn typed_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.replace(' ', "_"),
        Value::Number(n) => match n.as_f64() {
            // `1999`, not `1999.0`: numbers are parsed as floats either way.
            Some(f) if f.fract() == 0.0 && f.abs() < 1e15 => format!("{}", f as i64),
            _ => n.to_string(),
        },
        other => other.to_string(),
    }
}
