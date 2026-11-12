//! Local changes waiting to be pushed, in the order they were made.

use std::collections::HashSet;

use knoa_core::{
    Origin,
    protocol::{Entity, Mutation, new_id},
};
use rusqlite::{Connection, OptionalExtension as _, params};
use serde_json::{Map, Value, json};

use crate::Result;

/// What the outbox still holds for one row, so a pull doesn't undo it.
#[derive(Debug, Default)]
pub(crate) struct Pending {
    /// The row's last pending mutation deletes it.
    pub deleted: bool,
    /// Columns that pending `put`s set since the last pending delete.
    pub columns: HashSet<String>,
}

impl Pending {
    pub fn is_empty(&self) -> bool {
        !self.deleted && self.columns.is_empty()
    }
}

pub(crate) fn put(
    conn: &Connection,
    entity: Entity,
    key: &str,
    actor: Origin,
    data: &Map<String, Value>,
) -> Result<()> {
    let data = serde_json::to_string(data)?;
    append(conn, entity, key, actor, "put", Some(&data))
}

pub(crate) fn delete(conn: &Connection, entity: Entity, key: &str, actor: Origin) -> Result<()> {
    append(conn, entity, key, actor, "delete", None)
}

fn append(
    conn: &Connection,
    entity: Entity,
    key: &str,
    actor: Origin,
    op: &str,
    data: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO outbox (mid, entity, key, op, actor, data) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![new_id(), entity.slug(), key, op, actor.slug(), data],
    )?;
    Ok(())
}

/// What a push request adds around its mutations: `{"mutations":[]}`.
const REQUEST_BYTES: usize = r#"{"mutations":[]}"#.len();

/// The oldest mutations, in the order they were made: at most `limit`, and a request of no
/// more than `max_bytes` of JSON, though never none while anything is waiting.
pub(crate) fn pending(conn: &Connection, limit: usize, max_bytes: usize) -> Result<Vec<Mutation>> {
    let mut stmt = conn.prepare_cached(
        "SELECT mid, entity, key, op, actor, data FROM outbox ORDER BY seq LIMIT ?1",
    )?;
    let rows = stmt.query_map([limit], |row| {
        let data: Option<String> = row.get(5)?;
        Ok((
            json!({
                "mid": row.get::<_, String>(0)?,
                "entity": row.get::<_, String>(1)?,
                "key": row.get::<_, String>(2)?,
                "op": row.get::<_, String>(3)?,
                "actor": row.get::<_, String>(4)?,
            }),
            data,
        ))
    })?;
    let mut batch = Vec::new();
    let mut bytes = REQUEST_BYTES;
    for row in rows {
        let (mut mutation, data) = row?;
        if let Some(data) = data {
            mutation["data"] = serde_json::from_str(&data)?;
        }
        let mutation: Mutation = serde_json::from_value(mutation)?;
        // Measured as it will be sent, with the comma before the next one.
        bytes += serde_json::to_string(&mutation)?.len() + 1;
        if !batch.is_empty() && bytes > max_bytes {
            break;
        }
        batch.push(mutation);
    }
    Ok(batch)
}

pub(crate) fn acknowledge(conn: &Connection, mids: &[String]) -> Result<()> {
    let mut stmt = conn.prepare_cached("DELETE FROM outbox WHERE mid = ?1")?;
    for mid in mids {
        stmt.execute([mid])?;
    }
    Ok(())
}

/// The entity and key of a mutation still in the outbox.
pub(crate) fn row_of(conn: &Connection, mid: &str) -> Result<Option<(Entity, String)>> {
    let row = conn
        .query_row(
            "SELECT entity, key FROM outbox WHERE mid = ?1",
            [mid],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?;
    Ok(row.and_then(|(entity, key)| Some((Entity::from_slug(&entity)?, key))))
}

pub(crate) fn pending_for(conn: &Connection, entity: Entity, key: &str) -> Result<Pending> {
    let mut stmt = conn.prepare_cached(
        "SELECT op, data FROM outbox WHERE entity = ?1 AND key = ?2 ORDER BY seq",
    )?;
    let rows = stmt.query_map(params![entity.slug(), key], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
    })?;
    let mut pending = Pending::default();
    for row in rows {
        match row? {
            (op, _) if op == "delete" => {
                pending.deleted = true;
                pending.columns.clear();
            }
            (_, data) => {
                pending.deleted = false;
                let data: Map<String, Value> =
                    serde_json::from_str(data.as_deref().unwrap_or("{}"))?;
                pending
                    .columns
                    .extend(data.into_iter().map(|(column, _)| column));
            }
        }
    }
    Ok(pending)
}

/// Whether any of what the outbox holds for a row was made by the user.
pub(crate) fn has_user_changes(conn: &Connection, entity: Entity, key: &str) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM outbox WHERE entity = ?1 AND key = ?2 AND actor = ?3)",
        params![entity.slug(), key, Origin::User.slug()],
        |row| row.get(0),
    )?)
}

/// Forgets what the outbox holds for a row the server deleted, or will refuse.
pub(crate) fn forget(conn: &Connection, entity: Entity, key: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM outbox WHERE entity = ?1 AND key = ?2",
        params![entity.slug(), key],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Store;

    #[test]
    fn a_push_stops_at_its_byte_budget_but_never_comes_out_empty() {
        let store = Store::open_in_memory().unwrap();
        for key in ["a", "b", "c"] {
            let mut data = Map::new();
            data.insert("bookmark_id".into(), Value::from(key));
            data.insert("text".into(), Value::from("x".repeat(1000)));
            put(&store.conn, Entity::BookmarkText, key, Origin::User, &data).unwrap();
        }

        assert_eq!(pending(&store.conn, 10, 3000).unwrap().len(), 2);
        assert_eq!(pending(&store.conn, 10, 10).unwrap().len(), 1);
    }

    #[test]
    fn a_push_stays_within_its_byte_budget_as_sent() {
        let store = Store::open_in_memory().unwrap();
        let tag = "x".repeat(1000);
        for bookmark in ["a", "b"] {
            let data = json!({ "bookmark_id": bookmark, "tag": tag, "origin": "user",
                               "excluded": false });
            let Value::Object(data) = data else {
                unreachable!()
            };
            let key = format!("{bookmark}/{tag}");
            put(&store.conn, Entity::BookmarkTag, &key, Origin::User, &data).unwrap();
        }

        let batch = pending(&store.conn, 10, 3000).unwrap();
        let request = serde_json::to_string(&json!({ "mutations": batch })).unwrap();
        assert!(request.len() <= 3000, "{} bytes", request.len());
    }
}
