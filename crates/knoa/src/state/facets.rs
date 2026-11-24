//! Counts over a set of bookmarks, for the suggestions and the refine row. Each set is
//! counted in one pass on the background executor, so typing only filters these lists.

use std::collections::HashMap;

use knoa_core::{Bookmark, FieldType, FolderId, FolderTree, Kind};
use serde_json::Value;

/// What the current results hold.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResultFacets {
    pub total: usize,
    /// Most first.
    pub tags: Vec<(String, usize)>,
}

impl ResultFacets {
    pub fn of(bookmarks: &[Bookmark]) -> ResultFacets {
        let mut tags = Counter::default();
        for bookmark in bookmarks {
            for (tag, _) in &bookmark.tags {
                tags.add(tag);
            }
        }
        ResultFacets {
            total: bookmarks.len(),
            tags: tags.most_first(),
        }
    }
}

/// What the search finds without its typed text, which every suggestion counts within.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BaseFacets {
    /// Most first.
    pub kinds: Vec<(Kind, usize)>,
    pub tags: Vec<(String, usize)>,
    pub sites: Vec<(String, usize)>,
    /// Bookmarks in each folder or a folder under it, each counted once.
    pub folders: HashMap<FolderId, usize>,
    /// Values of text and list fields, by field name, most first.
    pub fields: Vec<(String, Vec<(String, usize)>)>,
}

impl BaseFacets {
    pub fn of(bookmarks: &[Bookmark], tree: &FolderTree) -> BaseFacets {
        let mut kinds = [0usize; Kind::ALL.len()];
        let (mut tags, mut sites) = (Counter::default(), Counter::default());
        let mut folders: HashMap<FolderId, usize> = HashMap::new();
        let mut fields: HashMap<&'static str, Counter> = HashMap::new();
        let mut reached = Vec::new();
        for bookmark in bookmarks {
            if let Some(kind) = bookmark.kind
                && let Some(at) = Kind::ALL.iter().position(|k| *k == kind)
            {
                kinds[at] += 1;
            }
            for (tag, _) in &bookmark.tags {
                tags.add(tag);
            }
            if !bookmark.site.is_empty() {
                sites.add(&bookmark.site);
            }
            reached.clear();
            for link in &bookmark.folders {
                let mut at = tree.get(link.folder).map(|f| f.id);
                while let Some(id) = at {
                    reached.push(id);
                    at = tree.parent(id).map(|f| f.id);
                }
            }
            reached.sort_unstable();
            reached.dedup();
            for id in &reached {
                *folders.entry(*id).or_default() += 1;
            }
            count_fields(bookmark, &mut fields);
        }
        let mut kinds: Vec<(Kind, usize)> = Kind::ALL
            .into_iter()
            .zip(kinds)
            .filter(|(_, n)| *n > 0)
            .collect();
        kinds.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        let mut fields: Vec<_> = fields
            .into_iter()
            .filter(|(_, values)| !values.0.is_empty())
            .map(|(name, values)| (name.to_owned(), values.most_first()))
            .collect();
        fields.sort_by(|a, b| a.0.cmp(&b.0));
        BaseFacets {
            kinds,
            tags: tags.most_first(),
            sites: sites.most_first(),
            folders,
            fields,
        }
    }
}

/// Text and list values only: numbers, dates and durations are compared, not suggested.
fn count_fields(bookmark: &Bookmark, fields: &mut HashMap<&'static str, Counter>) {
    let Some(kind) = bookmark.kind else {
        return;
    };
    for (name, value) in &bookmark.fields {
        let Some(def) = kind.field(name) else {
            continue;
        };
        // Keyed by the definition's name, which lives for the whole run, so counting
        // allocates no key per bookmark.
        let counter = fields.entry(def.name.as_str()).or_default();
        match (def.ty, value) {
            (FieldType::Text, Value::String(text)) => counter.add(text),
            (FieldType::List, Value::Array(items)) => {
                for item in items.iter().filter_map(Value::as_str) {
                    counter.add(item);
                }
            }
            _ => {}
        }
    }
}

/// Counts strings, allocating only for the first of each.
#[derive(Default)]
struct Counter(HashMap<String, usize>);

impl Counter {
    fn add(&mut self, value: &str) {
        if value.is_empty() {
            return;
        }
        match self.0.get_mut(value) {
            Some(n) => *n += 1,
            None => {
                self.0.insert(value.to_owned(), 1);
            }
        }
    }

    /// Most first; ties by name, so the order is stable between loads.
    fn most_first(self) -> Vec<(String, usize)> {
        let mut counts: Vec<_> = self.0.into_iter().collect();
        counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        counts
    }
}
