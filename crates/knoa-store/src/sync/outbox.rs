//! Local changes waiting to be pushed, in the order they were made.

use std::collections::HashSet;

use knoa_core::{
    Origin,
    protocol::{Entity, Mutation, new_id},
};
use rusqlite::{Connection, params};
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

/// Room for a mutation's envelope around its data: ids, entity, op, actor, punctuation.
const ENVELOPE_BYTES: usize = 256;

/// The oldest mutations, in the order they were made: at most `limit`, and no more than
/// about `max_bytes` of JSON, though never none while anything is waiting.
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
    let mut bytes = ENVELOPE_BYTES;
    for row in rows {
        let (mut mutation, data) = row?;
        bytes += ENVELOPE_BYTES + data.as_ref().map_or(0, String::len);
        if !batch.is_empty() && bytes > max_bytes {
            break;
        }
        if let Some(data) = data {
            mutation["data"] = serde_json::from_str(&data)?;
        }
        batch.push(serde_json::from_value(mutation)?);
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

/// Forgets what the outbox holds for a row the server deleted.
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
}
