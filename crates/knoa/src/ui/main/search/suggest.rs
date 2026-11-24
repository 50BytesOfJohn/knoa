//! What the suggestion list offers for the typed text: "Search for", then kinds, folders,
//! tags, sites and field values that start with it, then the top bookmarks. Counts come
//! from the base facets, so every row narrows the view the tokens already make.

mod groups;

use std::ops::Range;

use groups::{field_rows, folder_rows, kind_rows, site_rows, tag_rows};

use knoa_core::{Bookmark, BookmarkId, Filter, FolderId, FolderTree, Place};

use crate::{
    i18n::t,
    state::{facets::BaseFacets, search::Search},
};

/// At most this many rows of kinds, folders, tags, sites and fields together.
const MAX_FILTER_ROWS: usize = 6;
const MAX_BOOKMARK_ROWS: usize = 3;
/// Text starting with `#` offers only tags, this many.
const MAX_TAG_ONLY_ROWS: usize = 6;

/// What choosing a row does.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Action {
    /// Locks the typed text in as a text token.
    SearchFor,
    Add(Filter),
    /// Scopes the search to the folder, as clicking it in the sidebar does.
    Folder(FolderId),
    Open(BookmarkId),
}

/// The heading a row sits under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Group {
    Kinds,
    Folders,
    Tags,
    Sites,
    Fields,
    Bookmarks,
}

impl Group {
    pub fn heading(self) -> String {
        match self {
            Group::Kinds => t!("main-suggest-kinds"),
            Group::Folders => t!("main-suggest-folders"),
            Group::Tags => t!("main-suggest-tags"),
            Group::Sites => t!("main-suggest-sites"),
            Group::Fields => t!("main-suggest-fields"),
            Group::Bookmarks => t!("main-suggest-bookmarks"),
        }
    }

    fn max_rows(self) -> usize {
        match self {
            Group::Tags => 3,
            _ => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Row {
    pub action: Action,
    /// `None` for "Search for", which has no heading.
    pub group: Option<Group>,
    pub label: String,
    /// The bytes of `label` the user typed, drawn plain; the rest is the completion.
    pub typed: Range<usize>,
    pub count: Option<usize>,
}

impl Row {
    /// A filter row, for ⇥: not "Search for" and not a bookmark.
    pub fn is_filter(&self) -> bool {
        matches!(self.action, Action::Add(_) | Action::Folder(_))
    }
}

/// The rows for `text`, typed within `search` at `place`. `results` are the current
/// results, which already include the text.
pub(super) fn suggestions(
    text: &str,
    search: &Search,
    place: Place,
    base: &BaseFacets,
    results: &[Bookmark],
    folders: &FolderTree,
) -> Vec<Row> {
    let text = text.trim();
    if text.is_empty() {
        return Vec::new();
    }
    if let Some(tag) = text.strip_prefix('#') {
        let needle = tag.to_lowercase();
        return tag_rows(&needle, search, base, MAX_TAG_ONLY_ROWS);
    }
    let needle = text.to_lowercase();
    let mut rows = vec![Row {
        action: Action::SearchFor,
        group: None,
        label: t!("main-suggest-search-for", text = text),
        typed: 0..0,
        count: Some(results.len()),
    }];
    // Each group is looked through only while the budget has room, so a long list of
    // field values isn't scanned once the tags have filled it.
    let groups: [&dyn Fn(usize) -> Vec<Row>; 5] = [
        &|budget| kind_rows(&needle, search, base, Group::Kinds.max_rows().min(budget)),
        &|budget| {
            folder_rows(
                &needle,
                place,
                base,
                folders,
                Group::Folders.max_rows().min(budget),
            )
        },
        &|budget| tag_rows(&needle, search, base, Group::Tags.max_rows().min(budget)),
        &|budget| site_rows(&needle, search, base, Group::Sites.max_rows().min(budget)),
        &|budget| field_rows(&needle, search, base, Group::Fields.max_rows().min(budget)),
    ];
    let mut budget = MAX_FILTER_ROWS;
    for group_rows in groups {
        if budget == 0 {
            break;
        }
        let found = group_rows(budget);
        budget -= found.len();
        rows.extend(found);
    }
    rows.extend(results.iter().take(MAX_BOOKMARK_ROWS).map(|bookmark| Row {
        action: Action::Open(bookmark.id),
        group: Some(Group::Bookmarks),
        label: bookmark.title.clone(),
        typed: 0..0,
        count: None,
    }));
    rows
}

#[cfg(test)]
mod tests {
    use knoa_core::{Folder, FolderColor};

    use super::*;
    use crate::state::search::Token;

    fn labels(rows: &[Row]) -> Vec<&str> {
        rows.iter().map(|row| row.label.as_str()).collect()
    }

    #[test]
    fn suggestions_leave_out_applied_and_empty_values() {
        let folder = Folder {
            id: FolderId(1),
            parent: None,
            name: "Pasta night".into(),
            description: String::new(),
            auto_assign: true,
            color: FolderColor::default(),
            icon: None,
            count: 4,
        };
        let folders = FolderTree::new(vec![folder]);
        let base = BaseFacets {
            tags: vec![
                ("pasta".into(), 3),
                ("pastry".into(), 2),
                ("soup".into(), 1),
            ],
            sites: vec![("pastebin.com".into(), 1)],
            ..BaseFacets::default()
        };
        let search = Search {
            tokens: vec![Token::Filter(Filter::Tag("pasta".into()))],
            typed: "#pas".into(),
        };

        let rows = suggestions("#pas", &search, Place::Everything, &base, &[], &folders);
        assert_eq!(labels(&rows), ["pastry"]);

        // The folder has no bookmarks within the current tokens, so it isn't offered.
        let rows = suggestions("pas", &search, Place::Everything, &base, &[], &folders);
        assert_eq!(
            labels(&rows),
            ["Search for “pas”", "pastry", "pastebin.com"]
        );
    }
}
