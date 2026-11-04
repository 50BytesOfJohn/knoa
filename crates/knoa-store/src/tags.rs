//! Tags on bookmarks.

use knoa_core::{BookmarkId, normalize_tag};
use rusqlite::params;

use crate::{Result, Store, search};

impl Store {
    pub fn add_tag(&mut self, bookmark: BookmarkId, tag: &str) -> Result<()> {
        let tag = normalize_tag(tag);
        if tag.is_empty() {
            return Ok(());
        }
        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM tag_exclusions WHERE bookmark_id = ?1 AND tag = ?2",
            params![bookmark.0, tag],
        )?;
        tx.execute(
            "INSERT INTO bookmark_tags VALUES (?1, ?2, 'user') ON CONFLICT DO UPDATE SET origin = 'user'",
            params![bookmark.0, tag],
        )?;
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
        search::reindex(&tx, bookmark.0)?;
        tx.commit()?;
        Ok(())
    }

    /// Replaces the AI's tags with `tags`, keeping user tags and skipping removed ones.
    pub(crate) fn replace_ai_tags(
        tx: &rusqlite::Transaction<'_>,
        bookmark: BookmarkId,
        tags: &[String],
    ) -> Result<()> {
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
        Ok(())
    }
}
