//! Turns a `Query` into SQL, and keeps the full-text index in step with the tables.

use knoa_core::{Comparison, FieldFilter, FolderId, Kind, Place, Query};
use rusqlite::{Connection, params, types::Value};

/// Column weights for BM25, in `bookmarks_fts` column order: a title match counts most.
const BM25_WEIGHTS: &str = "10.0, 2.0, 4.0, 5.0, 3.0, 1.0";

/// The `FROM … WHERE …` part of a query, with its parameters.
pub(crate) struct Filter {
    pub sql: String,
    pub params: Vec<Value>,
    pub ranked: bool,
    /// Soonest first, for the Events place.
    pub by_start: bool,
}

impl Filter {
    /// `subtree` is the selected folder and every folder under it, for a folder place.
    pub fn new(query: &Query, subtree: &[FolderId]) -> Filter {
        let mut filter = Filter {
            sql: String::from("FROM bookmarks b"),
            params: Vec::new(),
            ranked: false,
            by_start: query.place == Place::Events,
        };
        let mut conditions: Vec<String> = Vec::new();

        if let Some(expr) = match_expression(&query.text) {
            filter
                .sql
                .push_str(" JOIN bookmarks_fts ON bookmarks_fts.rowid = b.id");
            conditions.push("bookmarks_fts MATCH ?".into());
            filter.params.push(Value::Text(expr));
            filter.ranked = true;
        }
        if let Some(workspace) = query.workspace {
            conditions.push("b.workspace_id = ?".into());
            filter.params.push(Value::Integer(workspace.0));
        }
        conditions.push(
            match query.place {
                Place::Trash => "b.deleted_at IS NOT NULL",
                Place::SetAside => "b.deleted_at IS NULL AND b.set_aside IS NOT NULL",
                _ => "b.deleted_at IS NULL AND b.set_aside IS NULL",
            }
            .into(),
        );
        match query.place {
            Place::Everything | Place::SetAside | Place::Trash => {}
            Place::UpNext => conditions.push("b.up_next = 1".into()),
            Place::Unsorted => conditions.push(
                "NOT EXISTS (SELECT 1 FROM bookmark_folders bf WHERE bf.bookmark_id = b.id)".into(),
            ),
            Place::NotOrganized => conditions.push("b.status != 'organized'".into()),
            Place::Gallery => {
                let slots = vec!["?"; Kind::MEDIA.len()].join(", ");
                conditions.push(format!("b.kind IN ({slots})"));
                filter.params.extend(
                    Kind::MEDIA
                        .iter()
                        .map(|kind| Value::Text(kind.slug().into())),
                );
            }
            Place::Events => {
                conditions.push("b.kind = ?".into());
                filter.params.push(Value::Text(Kind::Event.slug().into()));
            }
            Place::Folder(_) => {
                conditions.push(
                    "EXISTS (SELECT 1 FROM bookmark_folders bf WHERE bf.bookmark_id = b.id
                             AND bf.folder_id IN (SELECT value FROM json_each(?)))"
                        .into(),
                );
                let ids: Vec<i64> = subtree.iter().map(|id| id.0).collect();
                filter
                    .params
                    .push(Value::Text(serde_json::json!(ids).to_string()));
            }
        }
        if let Some(kind) = query.kind {
            conditions.push("b.kind = ?".into());
            filter.params.push(Value::Text(kind.slug().into()));
        }
        for tag in &query.tags {
            conditions.push("EXISTS (SELECT 1 FROM bookmark_tags bt WHERE bt.bookmark_id = b.id AND bt.tag = ?)".into());
            filter.params.push(Value::Text(tag.clone()));
        }
        for site in &query.sites {
            conditions.push("(b.site = ? OR b.site LIKE '%.' || ?)".into());
            filter.params.push(Value::Text(site.clone()));
            filter.params.push(Value::Text(site.clone()));
        }
        for field in &query.fields {
            conditions.push(field_condition(field, &mut filter.params));
        }
        if !conditions.is_empty() {
            filter.sql.push_str(" WHERE ");
            filter.sql.push_str(&conditions.join(" AND "));
        }
        filter
    }

    pub fn order_by(&self) -> String {
        if self.by_start {
            "ORDER BY json_extract(b.fields, '$.starts') IS NULL, json_extract(b.fields, '$.starts'), b.id".into()
        } else if self.ranked {
            format!("ORDER BY bm25(bookmarks_fts, {BM25_WEIGHTS}), b.added_at DESC")
        } else {
            "ORDER BY b.added_at DESC, b.id DESC".into()
        }
    }
}

/// One `name>value` filter's SQL condition, pushing its parameters.
fn field_condition(field: &FieldFilter, params: &mut Vec<Value>) -> String {
    let path = "json_extract(b.fields, '$.' || ?)";
    params.push(Value::Text(field.name.clone()));
    match (field.op, &field.value) {
        (Comparison::Matches, serde_json::Value::Number(n)) => {
            params.push(Value::Real(n.as_f64().unwrap_or_default()));
            format!("{path} = ?")
        }
        (Comparison::Matches, value) => {
            params.push(Value::Text(
                value.as_str().unwrap_or_default().to_lowercase(),
            ));
            format!("lower({path}) LIKE '%' || ? || '%'")
        }
        (op, serde_json::Value::String(text)) => {
            // Dates and date-times compare by their prefix, so `starts>2026-12-01`
            // reads as "after that day" whatever time the value carries.
            params.push(Value::Integer(text.chars().count() as i64));
            params.push(Value::Text(text.clone()));
            format!("substr({path}, 1, ?) {} ?", sign(op))
        }
        (op, value) => {
            params.push(Value::Real(value.as_f64().unwrap_or_default()));
            format!("{path} {} ?", sign(op))
        }
    }
}

fn sign(op: Comparison) -> &'static str {
    if op == Comparison::Greater { ">" } else { "<" }
}

/// Every word must match, each as a prefix, so results narrow as the user types.
fn match_expression(text: &str) -> Option<String> {
    let terms: Vec<String> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(|word| format!("\"{}\"*", word.to_lowercase()))
        .collect();
    (!terms.is_empty()).then(|| terms.join(" "))
}

/// Rewrites a bookmark's row in the full-text index from its current data.
pub(crate) fn reindex(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM bookmarks_fts WHERE rowid = ?1", [id])?;
    conn.execute(
        "INSERT INTO bookmarks_fts (rowid, title, url, summary, tags, details, body)
         SELECT b.id, b.title, b.url || ' ' || b.site, coalesce(b.summary, ''),
                coalesce((SELECT group_concat(tag, ' ') FROM bookmark_tags WHERE bookmark_id = b.id), ''),
                coalesce(b.kind, '') || ' ' || b.fields || ' ' || b.extras || ' ' || coalesce(b.browser_folder, ''),
                coalesce(b.page_text, '')
         FROM bookmarks b WHERE b.id = ?1",
        params![id],
    )?;
    Ok(())
}

pub(crate) fn kind_from_column(value: Option<&str>) -> Option<Kind> {
    value.and_then(Kind::parse)
}

#[cfg(test)]
mod tests {
    use knoa_core::{Enrichment, Fields};
    use serde_json::json;

    use crate::{
        NewBookmarks, Store,
        test_support::{WORKSPACE, bookmark},
    };

    use super::*;

    fn organize(store: &mut Store, kind: Kind, fields: serde_json::Value) {
        let job = store.take_jobs(0, 1).unwrap().remove(0);
        let fields: Fields = serde_json::from_value(fields).unwrap();
        let enrichment = Enrichment {
            kind,
            title: None,
            summary: None,
            fields,
            extras: Vec::new(),
            tags: Vec::new(),
            folders: Vec::new(),
            set_aside: None,
            model: "test".into(),
        };
        store
            .finish_organized(job.bookmark.id, &enrichment, 3)
            .unwrap();
    }

    #[test]
    fn events_holds_only_events_soonest_first() {
        let mut store = Store::open_in_memory().unwrap();
        let organized = [
            (
                "https://lu.ma/late",
                "Late gig",
                Kind::Event,
                json!({"starts": "2026-12-20T21:00"}),
            ),
            (
                "https://lu.ma/tbd",
                "Undated meetup",
                Kind::Event,
                json!({}),
            ),
            (
                "https://venue.example",
                "The venue",
                Kind::Place,
                json!({"city": "Berlin"}),
            ),
            (
                "https://lu.ma/soon",
                "Soon talk",
                Kind::Event,
                json!({"starts": "2026-12-02"}),
            ),
        ];
        for (n, (url, title, kind, fields)) in organized.into_iter().enumerate() {
            let link = [bookmark(url, title, n as i64)];
            store
                .import(WORKSPACE, "test", &link, NewBookmarks::Organize)
                .unwrap();
            organize(&mut store, kind, fields);
        }

        let events = |input: &str| -> Vec<String> {
            let query = Query::parse(input, Some(WORKSPACE), Place::Events);
            let found = store.search(&query, 10).unwrap();
            found.into_iter().map(|b| b.title).collect()
        };
        assert_eq!(events(""), ["Soon talk", "Late gig", "Undated meetup"]);
        assert_eq!(events("starts>2026-12-02"), ["Late gig"]);
        assert_eq!(store.counts(WORKSPACE).unwrap().events, 3);
    }
}
