//! What this Mac does for a linked library besides pushing and pulling: handing its
//! organizing queue to the cloud organizer, and moving thumbnails to and from the server.
//! None of it is recorded in the outbox; the server learns about it its own way.

use std::path::PathBuf;

use knoa_core::{BookmarkId, Image};
use rusqlite::params;

use crate::{Result, Store, jobs::drop_settled_backfills};

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

/// Only evidence from the server permits work that needs the remote row.
const ON_SERVER: &str = "b.sync_id IS NOT NULL AND b.delivery = 'confirmed'";

/// The UTC month of the unix time bound to `?2`, as `jobs.refused_month` stores it.
const MONTH: &str = "strftime('%Y-%m', ?2, 'unixepoch')";

impl Store {
    /// Releases blocked bookmarks from the queue when the cloud organizes this library.
    /// Returns whether any were.
    pub fn release_blocked_jobs(&mut self) -> Result<bool> {
        let tx = self.write()?;
        let restored = tx.execute(
            "UPDATE bookmarks SET status = CASE WHEN organized_at IS NULL THEN 'new' ELSE 'organized' END
             WHERE status = 'queued' AND delivery IN ('blocked_quota', 'blocked_invalid')",
            [],
        )?;
        let released = tx.execute(
            "DELETE FROM jobs WHERE bookmark_id IN
             (SELECT id FROM bookmarks WHERE delivery IN ('blocked_quota', 'blocked_invalid'))",
            [],
        )?;
        tx.commit()?;
        Ok(restored + released > 0)
    }

    /// Up to `limit` jobs of this Mac's queue for bookmarks the server already has. A job
    /// the cloud refused waits for the next UTC month after `now`.
    pub fn cloud_jobs(&mut self, limit: usize, now: i64) -> Result<Vec<CloudJob>> {
        let tx = self.write()?;
        drop_settled_backfills(&tx)?;
        let jobs = {
            let mut stmt = tx.prepare(&format!(
                "SELECT b.sync_id, j.folders_only FROM jobs j JOIN bookmarks b ON b.id = j.bookmark_id
                 WHERE (j.refused_month IS NULL AND b.status = 'queued' OR j.refused_month < {MONTH})
                   AND {ON_SERVER} LIMIT ?1"
            ))?;
            stmt.query_map(params![limit, now], |row| {
                Ok(CloudJob {
                    bookmark: row.get(0)?,
                    folders_only: row.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?
        };
        tx.commit()?;
        Ok(jobs)
    }

    /// The cloud organizer answered a hand-over: the jobs it accepted leave this Mac's
    /// queue, the ones it refused stay, marked with the month of `now`. Either way the
    /// status goes back to what it was until the server's own `queued` arrives in a pull.
    pub fn hand_to_cloud(
        &mut self,
        accepted: &[String],
        refused: &[String],
        now: i64,
    ) -> Result<()> {
        let tx = self.write()?;
        for key in accepted {
            tx.execute(
                "DELETE FROM jobs WHERE bookmark_id = (SELECT id FROM bookmarks WHERE sync_id = ?1)",
                [key],
            )?;
        }
        for key in refused {
            tx.execute(
                &format!(
                    "UPDATE jobs SET refused_month = {MONTH}
                     WHERE bookmark_id = (SELECT id FROM bookmarks WHERE sync_id = ?1)"
                ),
                params![key, now],
            )?;
        }
        for key in accepted.iter().chain(refused) {
            tx.execute(
                "UPDATE bookmarks SET status = CASE WHEN organized_at IS NULL THEN 'new' ELSE 'organized' END
                 WHERE sync_id = ?1 AND status = 'queued'",
                [key],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// The account organizes on this Mac now: jobs the cloud refused join the local queue
    /// again. Returns how many did.
    pub fn reclaim_refused_jobs(&mut self) -> Result<usize> {
        let tx = self.write()?;
        tx.execute(
            "UPDATE bookmarks SET status = 'queued'
             WHERE id IN (SELECT bookmark_id FROM jobs WHERE refused_month IS NOT NULL)",
            [],
        )?;
        let reclaimed = tx.execute(
            "UPDATE jobs SET refused_month = NULL WHERE refused_month IS NOT NULL",
            [],
        )?;
        tx.commit()?;
        Ok(reclaimed)
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
    use serde_json::{Value, json};

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
        let added = store
            .add_link(WORKSPACE, &crate::NewLink::new("https://a.dev/"), 10)
            .unwrap();
        assert_eq!(store.cloud_jobs(10, 0).unwrap().len(), 0);

        store.settle(&applied(&store)).unwrap();
        let jobs = store.cloud_jobs(10, 0).unwrap();
        assert_eq!(jobs.len(), 1);
        assert!(!jobs[0].folders_only);

        store
            .hand_to_cloud(&[jobs[0].bookmark.clone()], &[], 0)
            .unwrap();
        assert_eq!(store.cloud_jobs(10, 0).unwrap().len(), 0);
        assert!(store.take_jobs(i64::MAX, 10).unwrap().is_empty());
        let bookmark = store.bookmark(added.id).unwrap().unwrap();
        assert_eq!(bookmark.status, knoa_core::Status::New);
        // Handing over is this Mac's business: nothing goes to the server.
        assert_eq!(store.next_push().unwrap().len(), 0);
    }

    const URL: &str = "https://a.dev/";

    /// The last hour of November 2026 and the first of December, in UTC.
    const NOVEMBER: i64 = 1_796_079_600;
    const DECEMBER: i64 = 1_796_083_200;

    /// A bookmark this Mac saved, waiting to be handed to the cloud organizer.
    fn saved(store: &mut Store) -> (BookmarkId, String) {
        linked(store);
        let added = store
            .add_link(WORKSPACE, &crate::NewLink::new(URL), 10)
            .unwrap();
        store.settle(&applied(store)).unwrap();
        let key = store.cloud_jobs(10, NOVEMBER).unwrap().remove(0).bookmark;
        (added.id, key)
    }

    fn pull(store: &mut Store, key: &str, version: i64, edit: impl FnOnce(&mut Value)) {
        let mut row = bookmark_row(key, WORKSPACE_KEY, URL, "A");
        edit(&mut row);
        store
            .apply_remote(
                &[change(
                    json!({ "entity": "bookmark", "key": key, "version": version, "deleted": false, "data": row }),
                )],
                version,
            )
            .unwrap();
    }

    fn organized(row: &mut Value) {
        row["status"] = json!("organized");
        row["organized_at"] = json!(20);
    }

    #[test]
    fn a_result_pulled_before_the_hand_over_is_not_asked_for_again() {
        let mut store = Store::open_in_memory().unwrap();
        let (id, key) = saved(&mut store);
        pull(&mut store, &key, 2, organized);

        assert_eq!(store.cloud_jobs(10, NOVEMBER).unwrap(), []);
        assert!(store.take_jobs(i64::MAX, 10).unwrap().is_empty());
        let bookmark = store.bookmark(id).unwrap().unwrap();
        assert_eq!(bookmark.status, knoa_core::Status::Organized);
    }

    #[test]
    fn a_job_the_server_queued_at_push_does_not_run_here_after_a_flip_to_device() {
        let mut store = Store::open_in_memory().unwrap();
        let (id, key) = saved(&mut store);
        pull(&mut store, &key, 2, |row| row["status"] = json!("queued"));

        assert_eq!(store.reclaim_refused_jobs().unwrap(), 0);
        assert!(store.take_jobs(i64::MAX, 10).unwrap().is_empty());
        let bookmark = store.bookmark(id).unwrap().unwrap();
        assert_eq!(bookmark.status, knoa_core::Status::Queued);
    }

    #[test]
    fn organize_again_survives_a_pull_of_the_earlier_run() {
        let mut store = Store::open_in_memory().unwrap();
        let (id, key) = saved(&mut store);
        pull(&mut store, &key, 2, organized);
        store.queue(id).unwrap();

        pull(&mut store, &key, 3, |row| {
            organized(row);
            row["title"] = json!("Renamed elsewhere");
        });

        assert_eq!(store.cloud_jobs(10, NOVEMBER).unwrap().len(), 1);
    }

    #[test]
    fn a_refused_job_runs_here_once_the_account_organizes_on_this_mac() {
        let mut store = Store::open_in_memory().unwrap();
        let (id, key) = saved(&mut store);
        store
            .hand_to_cloud(&[], std::slice::from_ref(&key), NOVEMBER)
            .unwrap();
        assert!(store.take_jobs(i64::MAX, 10).unwrap().is_empty());

        assert_eq!(store.reclaim_refused_jobs().unwrap(), 1);

        let jobs = store.take_jobs(i64::MAX, 10).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].bookmark.id, id);
    }

    #[test]
    fn a_refused_job_is_handed_again_only_in_a_later_month() {
        let mut store = Store::open_in_memory().unwrap();
        let (_, key) = saved(&mut store);
        store
            .hand_to_cloud(&[], std::slice::from_ref(&key), NOVEMBER)
            .unwrap();

        assert_eq!(store.cloud_jobs(10, NOVEMBER).unwrap(), []);
        let next_month = store.cloud_jobs(10, DECEMBER).unwrap();
        assert_eq!(next_month.len(), 1);
        assert_eq!(next_month[0].bookmark, key);

        store
            .hand_to_cloud(std::slice::from_ref(&key), &[], DECEMBER)
            .unwrap();
        assert_eq!(store.cloud_jobs(10, DECEMBER).unwrap(), []);
    }

    #[test]
    fn a_thumbnail_replaced_during_its_upload_is_uploaded_in_turn() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let added = store
            .add_link(WORKSPACE, &crate::NewLink::new("https://a.dev/"), 10)
            .unwrap();
        store.settle(&applied(&store)).unwrap();
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
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let url = "https://a.dev/";
        let added = store
            .add_link(WORKSPACE, &crate::NewLink::new(url), 10)
            .unwrap();
        store.settle(&applied(&store)).unwrap();
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
