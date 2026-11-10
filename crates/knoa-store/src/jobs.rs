//! The durable organizing queue and saving its results.
//!
//! A job is a row in `jobs`. It stays there until the bookmark is organized or gives up,
//! so quitting the app mid-way loses nothing.

use knoa_core::{Bookmark, BookmarkId, Enrichment, FolderId, Page, WorkspaceId};
use rusqlite::{OptionalExtension as _, params};

use crate::{Result, Store, search};

/// A bookmark taken from the queue.
#[derive(Debug, Clone)]
pub struct Job {
    pub bookmark: Bookmark,
    pub attempts: u32,
    /// Only file the bookmark into folders again, from what it already has.
    pub folders_only: bool,
}

impl Store {
    /// Queues every bookmark in the workspace the AI hasn't organized, and returns how many
    /// were added.
    pub fn queue_not_organized(&mut self, workspace: WorkspaceId) -> Result<usize> {
        self.queue_matching(workspace, "status IN ('new', 'failed')", false)
    }

    /// Queues every bookmark in the workspace, organized or not, so the AI files them
    /// again with the current folders and instructions. Returns how many were queued.
    pub fn queue_all(&mut self, workspace: WorkspaceId) -> Result<usize> {
        self.queue_matching(workspace, "1", false)
    }

    /// Queues the workspace's organized bookmarks to be filed into folders again, without
    /// fetching or organizing them again. Bookmarks set aside get no folders, so they are
    /// left out. Returns how many were queued.
    pub fn queue_folders(&mut self, workspace: WorkspaceId) -> Result<usize> {
        self.queue_matching(
            workspace,
            "status = 'organized' AND set_aside IS NULL",
            true,
        )
    }

    /// Queues the workspace's bookmarks outside Trash that match `condition`, a SQL
    /// expression over `bookmarks` with no parameters.
    fn queue_matching(
        &mut self,
        workspace: WorkspaceId,
        condition: &str,
        folders_only: bool,
    ) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let ids: Vec<i64> = {
            let mut stmt = tx.prepare(&format!(
                "SELECT id FROM bookmarks WHERE workspace_id = ?1 AND deleted_at IS NULL AND {condition}"
            ))?;
            stmt.query_map([workspace.0], |row| row.get(0))?
                .collect::<rusqlite::Result<_>>()?
        };
        for id in &ids {
            enqueue(&tx, *id, folders_only)?;
        }
        tx.commit()?;
        Ok(ids.len())
    }

    /// Queues one bookmark, also when it was organized before (the detail panel's Organize).
    pub fn queue(&mut self, id: BookmarkId) -> Result<()> {
        let tx = self.conn.transaction()?;
        enqueue(&tx, id.0, false)?;
        tx.commit()?;
        Ok(())
    }

    /// Takes up to `limit` due jobs and marks their bookmarks as organizing.
    pub fn take_jobs(&mut self, now: i64, limit: usize) -> Result<Vec<Job>> {
        let tx = self.conn.transaction()?;
        let due: Vec<(i64, u32, bool)> = {
            let mut stmt = tx.prepare(
                "SELECT j.bookmark_id, j.attempts, j.folders_only FROM jobs j JOIN bookmarks b ON b.id = j.bookmark_id
                 WHERE j.run_after <= ?1 AND b.status = 'queued' ORDER BY b.added_at DESC LIMIT ?2",
            )?;
            stmt.query_map(params![now, limit], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .collect::<rusqlite::Result<_>>()?
        };
        for (id, ..) in &due {
            tx.execute(
                "UPDATE bookmarks SET status = 'organizing' WHERE id = ?1",
                [id],
            )?;
        }
        tx.commit()?;

        let mut jobs = Vec::with_capacity(due.len());
        for (id, attempts, folders_only) in due {
            if let Some(bookmark) = self.bookmark(BookmarkId(id))? {
                jobs.push(Job {
                    bookmark,
                    attempts,
                    folders_only,
                });
            }
        }
        Ok(jobs)
    }

    /// When the next waiting job becomes due, if any job is waiting.
    pub fn next_job_at(&self) -> Result<Option<i64>> {
        Ok(self
            .conn
            .query_row(
                "SELECT min(j.run_after) FROM jobs j JOIN bookmarks b ON b.id = j.bookmark_id WHERE b.status = 'queued'",
                [],
                |row| row.get(0),
            )
            .optional()?
            .flatten())
    }

    /// Jobs that were running when the app quit go back to the queue.
    pub fn requeue_interrupted(&mut self) -> Result<()> {
        self.conn.execute(
            "UPDATE bookmarks SET status = 'queued' WHERE status = 'organizing' AND id IN (SELECT bookmark_id FROM jobs)",
            [],
        )?;
        self.conn.execute(
            "UPDATE bookmarks SET status = 'new' WHERE status IN ('organizing', 'queued') AND id NOT IN (SELECT bookmark_id FROM jobs)",
            [],
        )?;
        Ok(())
    }

    /// Stores what the page fetch found. Values read from the page fill in kind, fields and
    /// summary until the AI provides its own, and the title of a link added by hand.
    pub fn save_page(&mut self, id: BookmarkId, page: &Page, now: i64) -> Result<()> {
        let tx = self.conn.transaction()?;
        let image = page.image.as_ref();
        tx.execute(
            "UPDATE bookmarks SET page_text = ?2, page_evidence = ?3, fetched_at = ?4,
                image_path = coalesce(?5, image_path), image_width = coalesce(?6, image_width),
                image_height = coalesce(?7, image_height),
                summary = CASE WHEN model IS NULL THEN coalesce(?8, summary) ELSE summary END,
                kind = CASE WHEN model IS NULL THEN coalesce(?9, kind) ELSE kind END,
                fields = CASE WHEN model IS NULL THEN ?10 ELSE fields END,
                title = CASE WHEN source IS NULL AND title = url THEN coalesce(?11, title) ELSE title END
             WHERE id = ?1",
            params![
                id.0,
                page.text,
                page.evidence,
                now,
                image.map(|i| i.path.to_string_lossy().into_owned()),
                image.map(|i| i.width),
                image.map(|i| i.height),
                page.description,
                page.kind.map(knoa_core::Kind::slug),
                serde_json::to_string(&page.fields)?,
                page.title,
            ],
        )?;
        search::reindex(&tx, id.0)?;
        tx.commit()?;
        Ok(())
    }

    /// Stores the AI's answer and takes the bookmark off the queue. A bookmark the user kept
    /// is never set aside. A bookmark deleted for good while it was organized is skipped.
    pub fn finish_organized(
        &mut self,
        id: BookmarkId,
        enrichment: &Enrichment,
        now: i64,
    ) -> Result<()> {
        let tx = self.conn.transaction()?;
        let updated = tx.execute(
            "UPDATE bookmarks SET kind = ?2, summary = ?3, fields = ?4, extras = ?5, model = ?6,
                status = 'organized', organized_at = ?7, error = NULL,
                set_aside = CASE WHEN kept = 1 THEN NULL ELSE ?8 END WHERE id = ?1",
            params![
                id.0,
                enrichment.kind.slug(),
                enrichment.summary,
                serde_json::to_string(&enrichment.fields)?,
                serde_json::to_string(&enrichment.extras)?,
                enrichment.model,
                now,
                enrichment.set_aside,
            ],
        )?;
        if updated == 0 {
            return Ok(());
        }
        Store::replace_ai_tags(&tx, id, &enrichment.tags)?;
        Store::add_ai_folders(&tx, id, &enrichment.folders)?;
        tx.execute("DELETE FROM jobs WHERE bookmark_id = ?1", [id.0])?;
        search::reindex(&tx, id.0)?;
        tx.commit()?;
        Ok(())
    }

    /// Adds the folders a folders-only job picked and takes the bookmark off the queue.
    /// Everything else it has stays as it was.
    pub fn finish_filed(&mut self, id: BookmarkId, folders: &[(FolderId, f32)]) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "UPDATE bookmarks SET status = 'organized', error = NULL WHERE id = ?1",
            [id.0],
        )?;
        Store::add_ai_folders(&tx, id, folders)?;
        tx.execute("DELETE FROM jobs WHERE bookmark_id = ?1", [id.0])?;
        tx.commit()?;
        Ok(())
    }

    /// Takes the bookmark off the queue without AI results: no AI is set up, so the page's
    /// own metadata is all it gets. It stays "not organized".
    pub fn finish_page_only(&mut self, id: BookmarkId) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("UPDATE bookmarks SET status = 'new' WHERE id = ?1", [id.0])?;
        tx.execute("DELETE FROM jobs WHERE bookmark_id = ?1", [id.0])?;
        tx.commit()?;
        Ok(())
    }

    /// Puts the job back to run again at `run_after`, counting the attempt.
    pub fn retry_job(&mut self, id: BookmarkId, error: &str, run_after: i64) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "UPDATE jobs SET attempts = attempts + 1, run_after = ?2 WHERE bookmark_id = ?1",
            params![id.0, run_after],
        )?;
        requeue(&tx, id, error)?;
        tx.commit()?;
        Ok(())
    }

    /// Puts a job back on the queue as it was, with the error shown on the bookmark. For
    /// failures that aren't the bookmark's fault, like a rejected API key: the attempt
    /// isn't counted, so the job keeps its retries for errors of its own.
    pub fn put_back_job(&mut self, id: BookmarkId, error: &str) -> Result<()> {
        let tx = self.conn.transaction()?;
        requeue(&tx, id, error)?;
        tx.commit()?;
        Ok(())
    }

    /// Gives up on a job. The bookmark keeps its title and URL and can be queued again. One
    /// whose job was `folders_only` only missed its folders, so it is still organized.
    ///
    /// The caller says what kind of job it was: Trash deletes the job's row even while the
    /// job runs, so the row can't tell.
    pub fn fail_job(&mut self, id: BookmarkId, error: &str, folders_only: bool) -> Result<()> {
        let status = if folders_only { "organized" } else { "failed" };
        let tx = self.conn.transaction()?;
        tx.execute(
            "UPDATE bookmarks SET error = ?2, status = ?3 WHERE id = ?1",
            params![id.0, error, status],
        )?;
        tx.execute("DELETE FROM jobs WHERE bookmark_id = ?1", [id.0])?;
        tx.commit()?;
        Ok(())
    }
}

/// Marks the bookmark queued again after its job ended. A bookmark trashed meanwhile lost
/// its job row, and nothing would ever take it, so it goes back to 'new' instead.
fn requeue(tx: &rusqlite::Transaction<'_>, id: BookmarkId, error: &str) -> Result<()> {
    tx.execute(
        "UPDATE bookmarks SET error = ?2, status = CASE
            WHEN EXISTS (SELECT 1 FROM jobs WHERE bookmark_id = ?1) THEN 'queued' ELSE 'new' END
         WHERE id = ?1",
        params![id.0, error],
    )?;
    Ok(())
}

fn enqueue(tx: &rusqlite::Transaction<'_>, id: i64, folders_only: bool) -> Result<()> {
    tx.execute(
        "UPDATE bookmarks SET status = 'queued' WHERE id = ?1 AND status != 'organizing'",
        [id],
    )?;
    tx.execute(
        "INSERT INTO jobs (bookmark_id, folders_only) VALUES (?1, ?2) ON CONFLICT DO UPDATE
         SET run_after = 0, folders_only = excluded.folders_only",
        params![id, folders_only],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use knoa_core::{Enrichment, Fields, Kind, Status};

    use super::*;
    use crate::{
        NewBookmarks,
        test_support::{WORKSPACE, bookmark},
    };

    #[test]
    fn queue_all_requeues_organized_bookmarks_but_not_trash() {
        let mut store = Store::open_in_memory().unwrap();
        let links = [
            bookmark("https://a.dev/", "A", 1),
            bookmark("https://b.dev/", "B", 2),
        ];
        store
            .import(WORKSPACE, "test", &links, NewBookmarks::Organize)
            .unwrap();
        let [trashed, kept] = <[Job; 2]>::try_from(store.take_jobs(0, 2).unwrap()).unwrap();
        for job in [&trashed, &kept] {
            store
                .finish_organized(job.bookmark.id, &enrichment(), 10)
                .unwrap();
        }
        store.trash(trashed.bookmark.id, 20).unwrap();

        assert_eq!(store.queue_all(WORKSPACE).unwrap(), 1);
        let requeued: Vec<_> = store
            .take_jobs(0, 10)
            .unwrap()
            .into_iter()
            .map(|job| job.bookmark.id)
            .collect();
        assert_eq!(requeued, [kept.bookmark.id]);
    }

    #[test]
    fn full_run_replaces_a_waiting_folders_only_job() {
        let mut store = Store::open_in_memory().unwrap();
        let links = [bookmark("https://a.dev/", "A", 1)];
        store
            .import(WORKSPACE, "test", &links, NewBookmarks::Organize)
            .unwrap();
        let [job] = <[Job; 1]>::try_from(store.take_jobs(0, 1).unwrap()).unwrap();
        store
            .finish_organized(job.bookmark.id, &enrichment(), 10)
            .unwrap();

        assert_eq!(store.queue_folders(WORKSPACE).unwrap(), 1);
        store.queue(job.bookmark.id).unwrap();
        let [job] = <[Job; 1]>::try_from(store.take_jobs(0, 1).unwrap()).unwrap();
        assert!(!job.folders_only);
    }

    #[test]
    fn a_bookmark_trashed_while_organizing_is_not_left_counted_after_a_retry() {
        let mut store = Store::open_in_memory().unwrap();
        let links = [bookmark("https://a.dev/", "A", 1)];
        store
            .import(WORKSPACE, "test", &links, NewBookmarks::Organize)
            .unwrap();
        let [job] = <[Job; 1]>::try_from(store.take_jobs(0, 1).unwrap()).unwrap();

        store.trash(job.bookmark.id, 10).unwrap();
        store.retry_job(job.bookmark.id, "timed out", 40).unwrap();

        assert_eq!(store.counts(WORKSPACE).unwrap().organizing, 0);
    }

    #[test]
    fn a_folders_only_job_trashed_and_then_failed_stays_organized() {
        let mut store = Store::open_in_memory().unwrap();
        let links = [bookmark("https://a.dev/", "A", 1)];
        store
            .import(WORKSPACE, "test", &links, NewBookmarks::Organize)
            .unwrap();
        let [job] = <[Job; 1]>::try_from(store.take_jobs(0, 1).unwrap()).unwrap();
        let id = job.bookmark.id;
        store.finish_organized(id, &enrichment(), 10).unwrap();
        store.queue_folders(WORKSPACE).unwrap();
        let [job] = <[Job; 1]>::try_from(store.take_jobs(0, 1).unwrap()).unwrap();

        store.trash(id, 20).unwrap();
        store.fail_job(id, "timed out", job.folders_only).unwrap();
        store.restore(id).unwrap();

        assert_eq!(
            store.bookmark(id).unwrap().unwrap().status,
            Status::Organized
        );
    }

    fn enrichment() -> Enrichment {
        Enrichment {
            kind: Kind::Article,
            summary: None,
            fields: Fields::new(),
            extras: Vec::new(),
            tags: Vec::new(),
            folders: Vec::new(),
            set_aside: None,
            model: "test".into(),
        }
    }
}
