//! What this Mac does for a linked library besides pushing and pulling: handing its
//! organizing queue to the cloud organizer, and moving thumbnails to and from the server.
//! None of it is recorded in the outbox; the server learns about it its own way.

use std::path::PathBuf;

use knoa_core::{BookmarkId, Image};
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
    pub pending_image: String,
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
    /// goes back to what it was until the server's own `queued` arrives in a pull. The
    /// server retains deferred requests and queues them as allowance becomes available.
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

    /// Up to `limit` local thumbnails waiting for upload.
    pub fn thumbnail_uploads(&self, limit: usize) -> Result<Vec<ThumbnailUpload>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT b.id, b.sync_id, b.image_path, b.image_width, b.image_height, b.pending_image FROM bookmarks b
             WHERE b.image_path IS NOT NULL AND b.pending_image IS NOT NULL AND b.deleted_at IS NULL
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
                pending_image: row.get(5)?,
            })
        })?;
        Ok(uploads.collect::<rusqlite::Result<_>>()?)
    }

    /// Up to `limit` server thumbnails missing from the cache, newest bookmarks first.
    pub fn thumbnail_downloads(&self, limit: usize) -> Result<Vec<ThumbnailDownload>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, image_key FROM bookmarks
             WHERE image_key IS NOT NULL AND image_path IS NULL AND pending_image IS NULL AND deleted_at IS NULL
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

    /// Caches a downloaded server thumbnail, unless a pull changed the bookmark's image
    /// while it was on its way.
    pub fn set_server_image(
        &mut self,
        id: BookmarkId,
        image_key: &str,
        image: &Image,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE bookmarks SET image_path = ?3, image_width = ?4, image_height = ?5
             WHERE id = ?1 AND image_key = ?2 AND pending_image IS NULL",
            params![
                id.0,
                image_key,
                image.path.to_string_lossy(),
                image.width,
                image.height
            ],
        )?;
        Ok(())
    }

    /// Records the key the server stored an uploaded thumbnail under, so the cached file
    /// counts as the server's image. A replacement keeps its pending token, so an older
    /// upload cannot acknowledge it, even if the cache reused the same path.
    pub fn set_image_key(
        &mut self,
        id: BookmarkId,
        pending_image: &str,
        image_key: &str,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE bookmarks SET image_key = ?3, pending_image = NULL
             WHERE id = ?1 AND pending_image = ?2",
            params![id.0, pending_image, image_key],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use knoa_core::{BookmarkId, Image};

    use crate::{Store, test_support::*};

    fn set_image(store: &mut Store, id: BookmarkId, image: &Image) {
        let bookmark = store.bookmark(id).unwrap().unwrap();
        let identity = store.bookmark_identity(&bookmark).unwrap().unwrap();
        store.set_image_if_current(&identity, image).unwrap();
    }

    #[test]
    fn queued_bookmarks_go_to_the_cloud_only_once_the_server_has_them() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let added = store.add_link(WORKSPACE, "https://a.dev/", 10).unwrap();
        assert_eq!(store.cloud_jobs(10).unwrap().len(), 0);

        let mids: Vec<String> = store
            .next_push()
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
        assert_eq!(store.next_push().unwrap().len(), 0);
    }

    #[test]
    fn a_thumbnail_replaced_during_its_upload_is_uploaded_in_turn() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let added = store.add_link(WORKSPACE, "https://a.dev/", 10).unwrap();
        let mids: Vec<String> = store
            .next_push()
            .unwrap()
            .into_iter()
            .map(|m| m.mid)
            .collect();
        store.acknowledge(&mids).unwrap();
        let image = |path: &str| Image {
            path: path.into(),
            width: 10,
            height: 10,
        };
        set_image(&mut store, added.id, &image("/cache/old.png"));
        let uploading = store.thumbnail_uploads(10).unwrap().remove(0);

        set_image(&mut store, added.id, &image("/cache/old.png"));
        store
            .set_image_key(added.id, &uploading.pending_image, "key-of-old")
            .unwrap();

        let next = store.thumbnail_uploads(10).unwrap();
        assert_eq!(next.len(), 1);
        assert_eq!(next[0].path, PathBuf::from("/cache/old.png"));
        assert_ne!(next[0].pending_image, uploading.pending_image);
    }

    #[test]
    fn a_pull_does_not_clear_a_pending_local_image() {
        use serde_json::json;

        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let url = "https://a.dev/";
        let added = store.add_link(WORKSPACE, url, 10).unwrap();
        let mids: Vec<_> = store
            .next_push()
            .unwrap()
            .into_iter()
            .map(|m| m.mid)
            .collect();
        store.acknowledge(&mids).unwrap();
        let image = Image {
            path: "/cache/new.png".into(),
            width: 10,
            height: 10,
        };
        set_image(&mut store, added.id, &image);
        let uploading = store.thumbnail_uploads(10).unwrap().remove(0);
        let mut row = bookmark_row(&uploading.bookmark, WORKSPACE_KEY, url, "A");
        for version in 2..=3 {
            row["image_key"] = json!(format!("old-server-key-{version}"));
            store
                .apply_remote(
                    &[change(
                        json!({ "entity": "bookmark", "key": uploading.bookmark,
                "version": version, "deleted": false, "data": row }),
                    )],
                    version,
                )
                .unwrap();
        }

        assert_eq!(
            store.thumbnail_uploads(10).unwrap(),
            vec![uploading.clone()]
        );
        store
            .set_image_key(added.id, &uploading.pending_image, "new-server-key")
            .unwrap();
        assert_eq!(store.thumbnail_uploads(10).unwrap(), []);
    }
}
