//! Tags on bookmarks.

use knoa_core::{BookmarkId, Origin, WorkspaceId, normalize_tag, tag_fits};
use rusqlite::params;

use crate::{Result, Store, search, sync::Recorder};

impl Store {
    /// Every tag on the workspace's bookmarks outside Trash, most used first.
    pub fn tags(&self, workspace: WorkspaceId) -> Result<Vec<(String, usize)>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT bt.tag, count(*) FROM bookmark_tags bt JOIN bookmarks b ON b.id = bt.bookmark_id
             WHERE b.workspace_id = ?1 AND b.deleted_at IS NULL
             GROUP BY bt.tag ORDER BY count(*) DESC, bt.tag",
        )?;
        let rows = stmt.query_map([workspace.0], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Adds a tag and lifts its removal, if any. The origin is the AI's only when putting
    /// back an AI tag that was removed by mistake.
    pub fn add_tag(&mut self, bookmark: BookmarkId, tag: &str, origin: Origin) -> Result<()> {
        let tag = normalize_tag(tag);
        // The server refuses a longer one, and the change would wait in the outbox for good.
        if !tag_fits(&tag) {
            return Ok(());
        }
        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM tag_exclusions WHERE bookmark_id = ?1 AND tag = ?2",
            params![bookmark.0, tag],
        )?;
        tx.execute(
            "INSERT INTO bookmark_tags VALUES (?1, ?2, ?3) ON CONFLICT DO UPDATE SET origin = ?3",
            params![bookmark.0, tag, origin.slug()],
        )?;
        Recorder::user(&tx)?.tag(bookmark.0, &tag)?;
        search::reindex(&tx, bookmark.0)?;
        tx.commit()?;
        Ok(())
    }

    /// Removes a tag and remembers it, so the AI doesn't add it back on a re-run.
    pub fn remove_tag(&mut self, bookmark: BookmarkId, tag: &str) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM bookmark_tags WHERE bookmark_id = ?1 AND tag = ?2",
            params![bookmark.0, tag],
        )?;
        tx.execute(
            "INSERT OR IGNORE INTO tag_exclusions VALUES (?1, ?2)",
            params![bookmark.0, tag],
        )?;
        Recorder::user(&tx)?.tag(bookmark.0, tag)?;
        search::reindex(&tx, bookmark.0)?;
        tx.commit()?;
        Ok(())
    }

    /// Replaces the AI's tags with `tags`, keeping user tags and skipping removed ones.
    pub(crate) fn replace_ai_tags(
        tx: &rusqlite::Transaction<'_>,
        recorder: &Recorder<'_>,
        bookmark: BookmarkId,
        tags: &[String],
    ) -> Result<()> {
        let before = recorder.tags_before(bookmark.0)?;
        tx.execute(
            "DELETE FROM bookmark_tags WHERE bookmark_id = ?1 AND origin = 'ai'",
            [bookmark.0],
        )?;
        for tag in tags {
            tx.execute(
                "INSERT INTO bookmark_tags (bookmark_id, tag, origin)
                 SELECT ?1, ?2, 'ai'
                 WHERE NOT EXISTS (SELECT 1 FROM tag_exclusions WHERE bookmark_id = ?1 AND tag = ?2)
                 ON CONFLICT DO NOTHING",
                params![bookmark.0, tag],
            )?;
        }
        recorder.tags_changed(bookmark.0, before)
    }
}

#[cfg(test)]
mod tests {
    use knoa_core::{Enrichment, Kind, Query};

    use super::*;
    use crate::{
        NewBookmarks,
        test_support::{WORKSPACE, bookmark},
    };

    #[test]
    fn a_tag_put_back_stays_the_ais_through_a_rerun() {
        let mut store = Store::open_in_memory().unwrap();
        store
            .import(
                WORKSPACE,
                "test",
                &[bookmark("https://a.dev/", "A", 1)],
                NewBookmarks::Organize,
            )
            .unwrap();
        let job = store.take_jobs(0, 1).unwrap().remove(0);
        let id = job.bookmark.id;
        let enrichment = Enrichment {
            kind: Kind::Article,
            summary: None,
            fields: knoa_core::Fields::new(),
            extras: Vec::new(),
            tags: vec!["rust".into()],
            folders: Vec::new(),
            set_aside: None,
            model: "test".into(),
        };
        store.finish_organized(id, &enrichment, 10).unwrap();
        store.remove_tag(id, "rust").unwrap();
        store.add_tag(id, "rust", Origin::Ai).unwrap();

        store.queue(id).unwrap();
        store.take_jobs(0, 1).unwrap();
        store.finish_organized(id, &enrichment, 20).unwrap();

        let saved = store.search(&Query::default(), 1).unwrap().remove(0);
        assert_eq!(saved.tags, [("rust".to_owned(), Origin::Ai)]);
    }
}
