//! What the library shows: a place, free text and typed filters, parsed from the search bar.

mod filter;

use serde_json::Value;

pub use filter::Filter;

use crate::{FolderId, Kind, WorkspaceId};

/// The sidebar selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Place {
    #[default]
    Everything,
    /// Bookmarks of a media kind, shown as pictures. Only on this Mac: it isn't part of
    /// the sync protocol.
    Gallery,
    /// Bookmarks of kind event, shown on a calendar or a list. Device-side like the
    /// Gallery: it isn't part of the sync protocol.
    Events,
    UpNext,
    /// Bookmarks in no folder.
    Unsorted,
    /// Bookmarks the AI hasn't organized yet.
    NotOrganized,
    /// Bookmarks the AI found don't fit the workspace's guideline. Every other place
    /// leaves them out.
    SetAside,
    /// Bookmarks the user deleted, until they are restored or deleted for good. Every other
    /// place leaves them out.
    Trash,
    Folder(FolderId),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Query {
    /// `None` searches every workspace.
    pub workspace: Option<WorkspaceId>,
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
    /// Parses search bar input. Words with known syntax become filters, the rest is text:
    /// `kind:movie`, `site:github.com`, `#noir`, `runtime>2h`, `year:1999`.
    pub fn parse(input: &str, workspace: Option<WorkspaceId>, place: Place) -> Query {
        let mut query = Query {
            workspace,
            place,
            ..Query::default()
        };
        let mut words = Vec::new();
        for word in input.split_whitespace() {
            match Filter::parse(word) {
                Some(filter) => query.apply(filter),
                None => words.push(word),
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

    /// Narrows the query by `filter`. A kind replaces the one already set.
    pub fn apply(&mut self, filter: Filter) {
        match filter {
            Filter::Kind(kind) => self.kind = Some(kind),
            Filter::Tag(tag) => self.tags.push(tag),
            Filter::Site(site) => self.sites.push(site),
            Filter::Field(field) => self.fields.push(field),
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
            None,
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
    fn typed_filters_parse_back() {
        let filters = [
            Filter::Kind(Kind::Recipe),
            Filter::Tag("pasta".into()),
            Filter::Site("youtube.com".into()),
            Filter::Field(FieldFilter {
                name: "director".into(),
                op: Comparison::Matches,
                value: json!("Christopher Nolan"),
            }),
            Filter::Field(FieldFilter {
                name: "runtime".into(),
                op: Comparison::Greater,
                value: json!(120),
            }),
            Filter::Field(FieldFilter {
                name: "year".into(),
                op: Comparison::Less,
                value: json!(1999.0),
            }),
        ];
        for filter in filters {
            assert_eq!(Filter::parse(&filter.typed()), Some(filter));
        }
    }

    #[test]
    fn unknown_prefixes_stay_text() {
        let query = Query::parse("http://example.com foo:bar", None, Place::Everything);
        assert_eq!(query.text, "http://example.com foo:bar");
    }
}
