//! What this Mac does for a linked library besides pushing and pulling: handing its
//! organizing queue to the cloud organizer, and moving thumbnails to and from the server.
//! None of it is recorded in the outbox; the server learns about it its own way.

use std::path::PathBuf;

use knoa_core::BookmarkId;
use rusqlite::params;

use crate::{Result, Store};

/// A queued bookmark the cloud organizer should take over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudJob {
    /// The bookmark's sync id.
    pub bookmark: String,
    pub folders_only: bool,
}

/// A thumbnail this Mac fetched that the server doesn't have yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThumbnailUpload {
    pub id: BookmarkId,
    /// The bookmark's sync id.
    pub bookmark: String,
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
}

/// A thumbnail the server has that isn't in this Mac's cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThumbnailDownload {
    pub id: BookmarkId,
    pub image_key: String,
}

/// Bookmarks the server has: linked, and with no creation still waiting in the outbox.
const ON_SERVER: &str = "b.sync_id IS NOT NULL
    AND NOT EXISTS (SELECT 1 FROM outbox o WHERE o.entity = 'bookmark' AND o.key = b.sync_id)";

impl Store {
    /// Up to `limit` jobs of this Mac's queue for bookmarks the server already has.
    pub fn cloud_jobs(&self, limit: usize) -> Result<Vec<CloudJob>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT b.sync_id, j.folders_only FROM jobs j JOIN bookmarks b ON b.id = j.bookmark_id
             WHERE b.status = 'queued' AND {ON_SERVER} LIMIT ?1"
        ))?;
        let jobs = stmt.query_map([limit], |row| {
            Ok(CloudJob {
                bookmark: row.get(0)?,
                folders_only: row.get(1)?,
            })
        })?;
        Ok(jobs.collect::<rusqlite::Result<_>>()?)
    }

    /// The cloud organizer took these bookmarks: they leave this Mac's queue. Their status
    /// goes back to what it was until the server's own `queued` arrives in a pull, so one
    /// the server didn't take (its monthly allowance ran out) doesn't wait forever.
    pub fn hand_to_cloud(&mut self, bookmarks: &[String]) -> Result<()> {
        let tx = self.conn.transaction()?;
        for key in bookmarks {
            tx.execute(
                "DELETE FROM jobs WHERE bookmark_id = (SELECT id FROM bookmarks WHERE sync_id = ?1)",
                [key],
            )?;
            tx.execute(
                "UPDATE bookmarks SET status = CASE WHEN organized_at IS NULL THEN 'new' ELSE 'organized' END
                 WHERE sync_id = ?1 AND status = 'queued'",
                [key],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Up to `limit` cached thumbnails the server has no image for.
    pub fn thumbnail_uploads(&self, limit: usize) -> Result<Vec<ThumbnailUpload>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT b.id, b.sync_id, b.image_path, b.image_width, b.image_height FROM bookmarks b
             WHERE b.image_path IS NOT NULL AND b.image_key IS NULL AND b.deleted_at IS NULL
               AND b.image_width IS NOT NULL AND b.image_height IS NOT NULL AND {ON_SERVER}
             ORDER BY b.added_at DESC LIMIT ?1"
        ))?;
        let uploads = stmt.query_map([limit], |row| {
            Ok(ThumbnailUpload {
                id: BookmarkId(row.get(0)?),
                bookmark: row.get(1)?,
                path: PathBuf::from(row.get::<_, String>(2)?),
                width: row.get(3)?,
                height: row.get(4)?,
            })
        })?;
        Ok(uploads.collect::<rusqlite::Result<_>>()?)
    }

    /// Up to `limit` server thumbnails missing from the cache, newest bookmarks first.
    pub fn thumbnail_downloads(&self, limit: usize) -> Result<Vec<ThumbnailDownload>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, image_key FROM bookmarks
             WHERE image_key IS NOT NULL AND image_path IS NULL AND deleted_at IS NULL
             ORDER BY added_at DESC LIMIT ?1",
        )?;
        let downloads = stmt.query_map([limit], |row| {
            Ok(ThumbnailDownload {
                id: BookmarkId(row.get(0)?),
                image_key: row.get(1)?,
            })
        })?;
        Ok(downloads.collect::<rusqlite::Result<_>>()?)
    }

    /// Records the key the server stored an uploaded thumbnail under, so the cached file
    /// counts as the server's image.
    pub fn set_image_key(&mut self, id: BookmarkId, image_key: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE bookmarks SET image_key = ?2 WHERE id = ?1",
            params![id.0, image_key],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Store, test_support::*};

    #[test]
    fn queued_bookmarks_go_to_the_cloud_only_once_the_server_has_them() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let added = store.add_link(WORKSPACE, "https://a.dev/", 10).unwrap();
        assert_eq!(store.cloud_jobs(10).unwrap().len(), 0);

        let mids: Vec<String> = store
            .pending(100)
            .unwrap()
            .into_iter()
            .map(|m| m.mid)
            .collect();
        store.acknowledge(&mids).unwrap();
        let jobs = store.cloud_jobs(10).unwrap();
        assert_eq!(jobs.len(), 1);
        assert!(!jobs[0].folders_only);

        store.hand_to_cloud(&[jobs[0].bookmark.clone()]).unwrap();
        assert_eq!(store.cloud_jobs(10).unwrap().len(), 0);
        assert!(store.take_jobs(i64::MAX, 10).unwrap().is_empty());
        let bookmark = store.bookmark(added.id).unwrap().unwrap();
        assert_eq!(bookmark.status, knoa_core::Status::New);
        // Handing over is this Mac's business: nothing goes to the server.
        assert_eq!(store.pending(100).unwrap().len(), 0);
    }
}
