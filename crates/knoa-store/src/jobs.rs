//! The durable organizing queue and saving its results.
//!
//! A job is a row in `jobs`. It stays there until the bookmark is organized or gives up,
//! so quitting the app mid-way loses nothing.

use knoa_core::{Bookmark, BookmarkId, Enrichment, Page, WorkspaceId};
use rusqlite::{OptionalExtension as _, params};

use crate::{Result, Store, search};

/// A bookmark taken from the queue.
#[derive(Debug, Clone)]
pub struct Job {
    pub bookmark: Bookmark,
    pub attempts: u32,
}

impl Store {
    /// Queues every bookmark in the workspace the AI hasn't organized, and returns how many
    /// were added.
    pub fn queue_not_organized(&mut self, workspace: WorkspaceId) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let ids: Vec<i64> = {
            let mut stmt = tx.prepare(
                "SELECT id FROM bookmarks WHERE workspace_id = ?1 AND status IN ('new', 'failed') AND deleted_at IS NULL",
            )?;
            stmt.query_map([workspace.0], |row| row.get(0))?
                .collect::<rusqlite::Result<_>>()?
        };
        for id in &ids {
            enqueue(&tx, *id)?;
        }
        tx.commit()?;
        Ok(ids.len())
    }

    /// Queues one bookmark, also when it was organized before (the detail panel's Organize).
    pub fn queue(&mut self, id: BookmarkId) -> Result<()> {
        let tx = self.conn.transaction()?;
        enqueue(&tx, id.0)?;
        tx.commit()?;
        Ok(())
    }

    /// Takes up to `limit` due jobs and marks their bookmarks as organizing.
    pub fn take_jobs(&mut self, now: i64, limit: usize) -> Result<Vec<Job>> {
        let tx = self.conn.transaction()?;
        let due: Vec<(i64, u32)> = {
            let mut stmt = tx.prepare(
                "SELECT j.bookmark_id, j.attempts FROM jobs j JOIN bookmarks b ON b.id = j.bookmark_id
                 WHERE j.run_after <= ?1 AND b.status = 'queued' ORDER BY b.added_at DESC LIMIT ?2",
            )?;
            stmt.query_map(params![now, limit], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<rusqlite::Result<_>>()?
        };
        for (id, _) in &due {
            tx.execute(
                "UPDATE bookmarks SET status = 'organizing' WHERE id = ?1",
                [id],
            )?;
        }
        tx.commit()?;

        let mut jobs = Vec::with_capacity(due.len());
        for (id, attempts) in due {
            if let Some(bookmark) = self.bookmark(BookmarkId(id))? {
                jobs.push(Job { bookmark, attempts });
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
        tx.execute(
            "UPDATE bookmarks SET status = 'queued', error = ?2 WHERE id = ?1",
            params![id.0, error],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Gives up on a job. The bookmark keeps its title and URL and can be queued again.
    pub fn fail_job(&mut self, id: BookmarkId, error: &str) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "UPDATE bookmarks SET status = 'failed', error = ?2 WHERE id = ?1",
            params![id.0, error],
        )?;
        tx.execute("DELETE FROM jobs WHERE bookmark_id = ?1", [id.0])?;
        tx.commit()?;
        Ok(())
    }
}

fn enqueue(tx: &rusqlite::Transaction<'_>, id: i64) -> Result<()> {
    tx.execute(
        "UPDATE bookmarks SET status = 'queued' WHERE id = ?1 AND status != 'organizing'",
        [id],
    )?;
    tx.execute(
        "INSERT INTO jobs (bookmark_id) VALUES (?1) ON CONFLICT DO UPDATE SET run_after = 0",
        [id],
    )?;
    Ok(())
}
