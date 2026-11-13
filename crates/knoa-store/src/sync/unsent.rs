//! Bookmarks the plan's bookmark limit kept off the server. Nothing is recorded for them
//! while they wait, and once the account has room they are sent again whole, with their
//! text, tags and folder links.

use knoa_core::protocol::{Entity, MutationResult, MutationStatus, RejectReason};
use rusqlite::{Connection, OptionalExtension as _, params};

use super::{
    ids::{self, Table},
    outbox, snapshot,
};
use crate::{Result, Store};

/// What the server's answers to a push left behind.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Settled {
    /// Bookmarks the plan's limit refused, now waiting on this Mac.
    pub parked: usize,
    /// Changes that broke the server's rules. Changes to a refused bookmark aren't
    /// counted: they go again with it.
    pub invalid: usize,
}

impl Store {
    /// Drops the answered mutations, as [`Store::acknowledge`], and keeps each bookmark
    /// the plan's limit refused until [`Store::resend_unsent`].
    pub fn settle(&mut self, results: &[MutationResult]) -> Result<Settled> {
        let tx = self.conn.transaction()?;
        let rows = results
            .iter()
            .map(|result| Ok((result, outbox::row_of(&tx, &result.mid)?)))
            .collect::<Result<Vec<_>>>()?;
        let mut settled = Settled::default();
        for (result, row) in &rows {
            if let (Some(RejectReason::Quota), Some((Entity::Bookmark, key))) = (result.reason, row)
                && park(&tx, key)?
            {
                settled.parked += 1;
            }
        }
        for (result, row) in &rows {
            let invalid = result.status == MutationStatus::Rejected
                && result.reason != Some(RejectReason::Quota);
            if invalid && !about_parked(&tx, row.as_ref())? {
                settled.invalid += 1;
            }
        }
        let mids: Vec<String> = results.iter().map(|result| result.mid.clone()).collect();
        outbox::acknowledge(&tx, &mids)?;
        tx.commit()?;
        Ok(settled)
    }

    /// How many bookmarks the plan's limit keeps on this Mac only.
    pub fn unsent_bookmarks(&self) -> Result<usize> {
        Ok(self
            .conn
            .query_row("SELECT count(*) FROM unsent_bookmarks", [], |row| {
                row.get(0)
            })?)
    }

    /// Queues every bookmark the plan's limit refused again, whole. Returns how many.
    pub fn resend_unsent(&mut self) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let ids = {
            let mut stmt = tx.prepare("SELECT bookmark_id FROM unsent_bookmarks")?;
            let ids = stmt.query_map([], |row| row.get::<_, i64>(0))?;
            ids.collect::<rusqlite::Result<Vec<_>>>()?
        };
        tx.execute("DELETE FROM unsent_bookmarks", [])?;
        for id in &ids {
            snapshot::record_bookmark_snapshot(&tx, *id)?;
        }
        tx.commit()?;
        Ok(ids.len())
    }
}

/// Whether the bookmark waits for room on the account.
pub(crate) fn is_unsent(conn: &Connection, bookmark: i64) -> Result<bool> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM unsent_bookmarks WHERE bookmark_id = ?1",
            [bookmark],
            |_| Ok(()),
        )
        .optional()?
        .is_some())
}

/// Protects retained local values with pending mutations before a pulled row is applied.
pub(super) fn resume(conn: &Connection, bookmark: i64) -> Result<bool> {
    let removed = conn.execute(
        "DELETE FROM unsent_bookmarks WHERE bookmark_id = ?1",
        [bookmark],
    )?;
    if removed == 0 {
        return Ok(false);
    }
    snapshot::record_bookmark_snapshot(conn, bookmark)?;
    Ok(true)
}

/// Keeps the bookmark known as `key` back, and forgets what the outbox still holds for
/// it, which the server would refuse too. False when there is no such bookmark.
fn park(conn: &Connection, key: &str) -> Result<bool> {
    let Some(id) = ids::local_id(conn, Table::Bookmarks, key)? else {
        return Ok(false);
    };
    conn.execute(
        "INSERT OR IGNORE INTO unsent_bookmarks (bookmark_id) VALUES (?1)",
        [id],
    )?;
    conn.execute(
        "DELETE FROM outbox
         WHERE (entity IN (?2, ?3) AND key = ?1)
            OR (entity IN (?4, ?5) AND substr(key, 1, length(?1) + 1) = ?1 || '/')",
        params![
            key,
            Entity::Bookmark.slug(),
            Entity::BookmarkText.slug(),
            Entity::BookmarkTag.slug(),
            Entity::BookmarkFolder.slug(),
        ],
    )?;
    Ok(true)
}

/// Whether an outbox row is about a parked bookmark.
fn about_parked(conn: &Connection, row: Option<&(Entity, String)>) -> Result<bool> {
    let Some((entity, key)) = row else {
        return Ok(false);
    };
    let bookmark = match entity {
        Entity::Bookmark | Entity::BookmarkText => key,
        Entity::BookmarkTag | Entity::BookmarkFolder => key.split('/').next().unwrap_or(key),
        Entity::Workspace | Entity::Folder => return Ok(false),
    };
    match ids::local_id(conn, Table::Bookmarks, bookmark)? {
        Some(id) => is_unsent(conn, id),
        None => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use knoa_core::{Origin, Page, Place, Query, protocol::bookmark_id};
    use serde_json::{Value, json};

    use super::*;
    use crate::{
        NewBookmarks,
        test_support::{
            WORKSPACE, WORKSPACE_KEY, bookmark, bookmark_row, change, import_one, linked, mids,
            pushed,
        },
    };

    fn answer(mutation: &Value, reason: &str) -> MutationResult {
        serde_json::from_value(json!({
            "mid": mutation["mid"], "status": "rejected", "reason": reason,
        }))
        .unwrap()
    }

    #[test]
    fn a_pulled_parked_bookmark_retains_local_edits_and_resumes_syncing() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let id = import_one(&mut store);
        let put = pushed(&store).remove(0);
        store.settle(&[answer(&put, "quota")]).unwrap();
        store.set_up_next(id, true).unwrap();
        store.add_tag(id, "rust", Origin::User).unwrap();
        let page = Page {
            text: "Local page text".into(),
            description: Some("Local summary".into()),
            ..Page::default()
        };
        store.save_page(id, &page, 10).unwrap();
        assert_eq!(pushed(&store), Vec::<Value>::new());
        let key = bookmark_id(WORKSPACE_KEY, "https://a.dev/").unwrap();
        let changes = [
            change(json!({
                "entity": "bookmark", "key": key, "version": 2, "deleted": false,
                "data": bookmark_row(&key, WORKSPACE_KEY, "https://a.dev/", "Remote title"),
            })),
            change(json!({
                "entity": "bookmark_tag", "key": format!("{key}/rust"),
                "version": 3, "deleted": false,
                "data": { "bookmark_id": key, "tag": "rust", "origin": "user", "excluded": true },
            })),
            change(json!({
                "entity": "bookmark_text", "key": key, "version": 4, "deleted": false,
                "data": { "bookmark_id": key, "text": "Remote text" },
            })),
        ];

        store.apply_remote(&changes, 4).unwrap();

        let saved = store.bookmark(id).unwrap().unwrap();
        assert_eq!(saved.title, "A");
        assert!(saved.up_next);
        assert_eq!(saved.summary, page.description);
        assert_eq!(saved.tags, [("rust".into(), Origin::User)]);
        let found = store
            .search(&Query::parse("local page text", None, Place::Everything), 1)
            .unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, id);
        assert_eq!(store.unsent_bookmarks().unwrap(), 0);
        let pending = pushed(&store);
        assert_eq!(pending.len(), 3);
        assert_eq!(pending[0]["data"]["title"], "A");
        assert_eq!(pending[0]["data"]["up_next"], true);
        store.acknowledge(&mids(&store)).unwrap();
        store.set_up_next(id, false).unwrap();
        let [edit] = <[Value; 1]>::try_from(pushed(&store)).unwrap();
        assert_eq!(edit["data"], json!({ "up_next": false }));
    }

    #[test]
    fn a_bookmark_past_the_plans_limit_goes_again_whole_when_resent() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        store
            .import(
                WORKSPACE,
                "helium:Default",
                &[bookmark("https://a.dev/", "A", 1)],
                NewBookmarks::Organize,
            )
            .unwrap();
        let id = store.search(&Query::default(), 1).unwrap()[0].id;
        store.add_tag(id, "rust", Origin::User).unwrap();
        let [put, tag] = <[Value; 2]>::try_from(pushed(&store)).unwrap();

        let settled = store
            .settle(&[answer(&put, "quota"), answer(&tag, "invalid")])
            .unwrap();
        store.add_tag(id, "web", Origin::User).unwrap();

        assert_eq!(
            settled,
            Settled {
                parked: 1,
                invalid: 0
            }
        );
        assert_eq!(pushed(&store), Vec::<Value>::new());
        assert_eq!(store.unsent_bookmarks().unwrap(), 1);

        assert_eq!(store.resend_unsent().unwrap(), 1);
        let entities: Vec<Value> = pushed(&store).iter().map(|m| m["entity"].clone()).collect();
        assert_eq!(
            entities,
            [
                json!("bookmark"),
                json!("bookmark_tag"),
                json!("bookmark_tag")
            ]
        );
        assert_eq!(store.unsent_bookmarks().unwrap(), 0);
    }
}
