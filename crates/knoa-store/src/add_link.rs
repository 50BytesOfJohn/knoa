//! Links the user adds by hand.

use knoa_core::{BookmarkId, WorkspaceId, site_of};
use rusqlite::{OptionalExtension as _, params};

use crate::{Result, Store, search, sync::Recorder, trash};

/// A link to add by hand, with what the browser extension knows about its page.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NewLink<'a> {
    pub url: &'a str,
    /// The tab's title, used instead of the URL. Blank titles are ignored.
    pub title: Option<&'a str>,
    /// The page as the user saw it. The organizer reads it instead of fetching the page,
    /// so pages behind a login get organized too. It stays on this Mac, with the job.
    pub snapshot: Option<&'a str>,
}

impl<'a> NewLink<'a> {
    pub fn new(url: &'a str) -> NewLink<'a> {
        NewLink {
            url,
            ..NewLink::default()
        }
    }
}

/// A link the user added by hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddedLink {
    pub id: BookmarkId,
    pub outcome: AddOutcome,
}

/// What adding a link did to the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddOutcome {
    /// A new bookmark, queued for organizing.
    Added,
    /// The workspace had it in Trash; it came back as it was.
    Restored,
    /// The workspace already had it.
    Existing,
}

impl Store {
    /// Adds a link by hand and queues it for organizing. The user chose it, so the
    /// workspace's guideline never sets it aside, and a deleted copy comes back. Without
    /// a title, the URL stands in as one until the page is fetched. A link the workspace
    /// already has isn't queued again, and its title and snapshot are dropped.
    pub fn add_link(
        &mut self,
        workspace: WorkspaceId,
        link: &NewLink<'_>,
        now: i64,
    ) -> Result<AddedLink> {
        let url = link.url;
        let tx = self.conn.transaction()?;
        let existing: Option<(i64, bool)> = tx
            .query_row(
                "SELECT id, deleted_at IS NOT NULL FROM bookmarks WHERE workspace_id = ?1 AND url = ?2",
                params![workspace.0, url],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let recorder = Recorder::user(&tx)?;
        if let Some((id, deleted)) = existing {
            recorder.on_bookmark(id, || {
                tx.execute(
                    "UPDATE bookmarks SET kept = 1, set_aside = NULL, deleted_at = NULL WHERE id = ?1",
                    [id],
                )?;
                Ok(())
            })?;
            tx.commit()?;
            let outcome = if deleted {
                AddOutcome::Restored
            } else {
                AddOutcome::Existing
            };
            return Ok(AddedLink {
                id: BookmarkId(id),
                outcome,
            });
        }
        trash::forget_deleted_url(&tx, workspace, url)?;
        let title = link
            .title
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .unwrap_or(url);
        tx.execute(
            "INSERT INTO bookmarks (workspace_id, url, title, site, added_at, status, kept)
             VALUES (?1, ?2, ?3, ?4, ?5, 'queued', 1)",
            params![workspace.0, url, title, site_of(url), now],
        )?;
        let id = tx.last_insert_rowid();
        tx.execute(
            "INSERT INTO jobs (bookmark_id, snapshot) VALUES (?1, ?2)",
            params![id, link.snapshot],
        )?;
        recorder.created_bookmark(id)?;
        search::reindex(&tx, id)?;
        tx.commit()?;
        Ok(AddedLink {
            id: BookmarkId(id),
            outcome: AddOutcome::Added,
        })
    }
}

#[cfg(test)]
mod tests {
    use knoa_core::{Enrichment, Kind, Place, Query};

    use super::*;
    use crate::{
        Job,
        test_support::{WORKSPACE, linked, pushed},
    };

    #[test]
    fn a_kept_link_is_never_set_aside() {
        let mut store = Store::open_in_memory().unwrap();
        let added = store
            .add_link(WORKSPACE, &NewLink::new("https://a.dev/"), 1)
            .unwrap();
        let job = store.take_jobs(1, 1).unwrap().remove(0);
        assert_eq!(job.bookmark.id, added.id);
        store.finish_organized(added.id, &enrichment(), 2).unwrap();
        let set_aside = Query {
            place: Place::SetAside,
            ..Query::default()
        };
        assert_eq!(store.count(&set_aside).unwrap(), 0);
        assert_eq!(store.count(&Query::default()).unwrap(), 1);
    }

    #[test]
    fn a_snapshot_goes_to_the_organizer_only_and_leaves_with_the_job() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let link = NewLink {
            url: "https://a.dev/",
            title: Some(" Inbox - A "),
            snapshot: Some("<p>private page</p>"),
        };
        let added = store.add_link(WORKSPACE, &link, 1).unwrap();
        let again = store.add_link(WORKSPACE, &link, 2).unwrap();

        assert_eq!(again.outcome, AddOutcome::Existing);
        let [job] = <[Job; 1]>::try_from(store.take_jobs(2, 10).unwrap()).unwrap();
        assert_eq!(job.bookmark.title, "Inbox - A");
        assert_eq!(job.snapshot.as_deref(), Some("<p>private page</p>"));
        let outbox = serde_json::to_string(&pushed(&store)).unwrap();
        assert!(outbox.contains("Inbox - A"));
        assert!(!outbox.contains("private page"), "{outbox}");

        store.finish_organized(added.id, &enrichment(), 3).unwrap();
        let snapshots: i64 = store
            .conn
            .query_row("SELECT count(snapshot) FROM jobs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(snapshots, 0);
    }

    fn enrichment() -> Enrichment {
        Enrichment {
            kind: Kind::Article,
            title: None,
            summary: None,
            fields: knoa_core::Fields::new(),
            extras: Vec::new(),
            tags: Vec::new(),
            folders: Vec::new(),
            set_aside: Some("not about work".into()),
            model: "test".into(),
        }
    }
}
