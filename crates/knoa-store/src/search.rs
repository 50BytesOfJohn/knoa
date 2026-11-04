//! Turns a `Query` into SQL, and keeps the full-text index in step with the tables.

use knoa_core::{Comparison, Kind, Place, Query};
use rusqlite::{Connection, params, types::Value};

/// Column weights for BM25, in `bookmarks_fts` column order: a title match counts most.
const BM25_WEIGHTS: &str = "10.0, 2.0, 4.0, 5.0, 3.0, 1.0";

/// The `FROM … WHERE …` part of a query, with its parameters.
pub(crate) struct Filter {
    pub sql: String,
    pub params: Vec<Value>,
    pub ranked: bool,
}

impl Filter {
    pub fn new(query: &Query) -> Filter {
        let mut filter = Filter {
            sql: String::from("FROM bookmarks b"),
            params: Vec::new(),
            ranked: false,
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
        if query.place == Place::SetAside {
            conditions.push("b.set_aside IS NOT NULL".into());
        } else {
            conditions.push("b.set_aside IS NULL".into());
        }
        match query.place {
            Place::Everything | Place::SetAside => {}
            Place::UpNext => conditions.push("b.up_next = 1".into()),
            Place::Unsorted => conditions.push(
                "NOT EXISTS (SELECT 1 FROM bookmark_folders bf WHERE bf.bookmark_id = b.id)".into(),
            ),
            Place::NotOrganized => conditions.push("b.status != 'organized'".into()),
            Place::Folder(folder) => {
                conditions.push("EXISTS (SELECT 1 FROM bookmark_folders bf WHERE bf.bookmark_id = b.id AND bf.folder_id = ?)".into());
                filter.params.push(Value::Integer(folder.0));
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
            let path = "json_extract(b.fields, '$.' || ?)";
            filter.params.push(Value::Text(field.name.clone()));
            let condition = match (field.op, &field.value) {
                (Comparison::Matches, serde_json::Value::Number(n)) => {
                    filter
                        .params
                        .push(Value::Real(n.as_f64().unwrap_or_default()));
                    format!("{path} = ?")
                }
                (Comparison::Matches, value) => {
                    filter.params.push(Value::Text(
                        value.as_str().unwrap_or_default().to_lowercase(),
                    ));
                    format!("lower({path}) LIKE '%' || ? || '%'")
                }
                (op, value) => {
                    filter
                        .params
                        .push(Value::Real(value.as_f64().unwrap_or_default()));
                    let sign = if op == Comparison::Greater { ">" } else { "<" };
                    format!("{path} {sign} ?")
                }
            };
            conditions.push(condition);
        }
        if !conditions.is_empty() {
            filter.sql.push_str(" WHERE ");
            filter.sql.push_str(&conditions.join(" AND "));
        }
        filter
    }

    pub fn order_by(&self) -> String {
        if self.ranked {
            format!("ORDER BY bm25(bookmarks_fts, {BM25_WEIGHTS}), b.added_at DESC")
        } else {
            "ORDER BY b.added_at DESC, b.id DESC".into()
        }
    }
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
