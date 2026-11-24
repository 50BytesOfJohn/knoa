//! Each group's rows: the values of one facet that start with the typed text, with
//! their counts, leaving out what the search already has.

use std::ops::Range;

use knoa_core::{Comparison, FieldFilter, Filter, FolderTree, Kind, Place};
use serde_json::Value;

use super::{Action, Group, Row};
use crate::{
    i18n::{self, t},
    state::{facets::BaseFacets, search::Search},
};

pub(super) fn kind_rows(needle: &str, search: &Search, base: &BaseFacets, max: usize) -> Vec<Row> {
    let applied = search.kind();
    base.kinds
        .iter()
        .filter(|(kind, n)| *n > 0 && Some(*kind) != applied)
        .filter_map(|&(kind, count)| {
            let plural = i18n::kind_plural(kind);
            let typed = prefix(&plural, needle)
                .or_else(|| prefix(&i18n::kind_label(kind), needle).map(|_| 0..0))?;
            Some(filter_row(
                Action::Add(Filter::Kind(kind)),
                Group::Kinds,
                plural,
                typed,
                count,
            ))
        })
        .take(max)
        .collect()
}

/// Folders whose name, or any word in it, starts with the text.
pub(super) fn folder_rows(
    needle: &str,
    place: Place,
    base: &BaseFacets,
    folders: &FolderTree,
    max: usize,
) -> Vec<Row> {
    folders
        .folders()
        .iter()
        .filter(|folder| place != Place::Folder(folder.id))
        .filter_map(|folder| {
            let count = base.folders.get(&folder.id).copied().filter(|n| *n > 0)?;
            let typed = word_prefix(&folder.name, needle)?;
            let action = Action::Folder(folder.id);
            Some(filter_row(
                action,
                Group::Folders,
                folder.name.clone(),
                typed,
                count,
            ))
        })
        .take(max)
        .collect()
}

pub(super) fn tag_rows(needle: &str, search: &Search, base: &BaseFacets, max: usize) -> Vec<Row> {
    base.tags
        .iter()
        .filter(|(tag, n)| *n > 0 && !search.has_tag(tag))
        .filter_map(|(tag, count)| {
            let typed = prefix(tag, needle)?;
            let action = Action::Add(Filter::Tag(tag.clone()));
            Some(filter_row(action, Group::Tags, tag.clone(), typed, *count))
        })
        .take(max)
        .collect()
}

pub(super) fn site_rows(needle: &str, search: &Search, base: &BaseFacets, max: usize) -> Vec<Row> {
    base.sites
        .iter()
        .filter(|(site, n)| *n > 0 && !search.has_site(site))
        .filter_map(|(site, count)| {
            let typed = prefix(site, needle)?;
            let action = Action::Add(Filter::Site(site.clone()));
            Some(filter_row(
                action,
                Group::Sites,
                site.clone(),
                typed,
                *count,
            ))
        })
        .take(max)
        .collect()
}

/// "Director: Christopher Nolan", for values of text and list fields.
pub(super) fn field_rows(needle: &str, search: &Search, base: &BaseFacets, max: usize) -> Vec<Row> {
    let mut rows = Vec::new();
    for (name, values) in &base.fields {
        let Some(def) = Kind::ALL.into_iter().find_map(|kind| kind.field(name)) else {
            continue;
        };
        for (value, count) in values {
            let Some(typed) = prefix(value, needle) else {
                continue;
            };
            let filter = Filter::Field(FieldFilter {
                name: name.clone(),
                op: Comparison::Matches,
                value: Value::from(value.as_str()),
            });
            if *count == 0 || search.contains(&filter) {
                continue;
            }
            let field = i18n::field_label(def);
            let label = t!(
                "main-suggest-field",
                field = field,
                value = (value.as_str())
            );
            let start = label.rfind(value.as_str()).unwrap_or_default();
            let typed = start + typed.start..start + typed.end;
            rows.push(filter_row(
                Action::Add(filter),
                Group::Fields,
                label,
                typed,
                *count,
            ));
            if rows.len() == max {
                return rows;
            }
        }
    }
    rows
}

fn filter_row(
    action: Action,
    group: Group,
    label: String,
    typed: Range<usize>,
    count: usize,
) -> Row {
    Row {
        action,
        group: Some(group),
        label,
        typed,
        count: Some(count),
    }
}

/// Where `needle` (lowercase) starts `label`, as bytes of `label`.
fn prefix(label: &str, needle: &str) -> Option<Range<usize>> {
    let mut chars = label.char_indices();
    for wanted in needle.chars() {
        let (_, ch) = chars.next()?;
        if !ch.to_lowercase().eq(wanted.to_lowercase()) {
            return None;
        }
    }
    let end = chars.next().map_or(label.len(), |(i, _)| i);
    Some(0..end)
}

/// [`prefix`] at the start of any word of `label`.
fn word_prefix(label: &str, needle: &str) -> Option<Range<usize>> {
    let mut starts = std::iter::once(0).chain(
        label
            .char_indices()
            .filter(|(_, ch)| ch.is_whitespace())
            .map(|(i, ch)| i + ch.len_utf8()),
    );
    starts.find_map(|start| {
        prefix(&label[start..], needle).map(|typed| start + typed.start..start + typed.end)
    })
}
