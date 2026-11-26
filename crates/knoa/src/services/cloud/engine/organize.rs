//! Handing this Mac's organizing queue to the cloud organizer, when the account has it.

use gpui::AsyncApp;
use knoa_core::protocol::{MAX_PUSH, OrganizeRequest, OrganizeScope};
use knoa_store::Store;

use super::{Engine, Failure, net};
use crate::{
    services::{
        cloud::Wake,
        organizer::{Organizer, now},
    },
    state::library::Library,
};

impl Engine {
    /// Asks the cloud organizer to take this Mac's queue: new bookmarks, and "Organize
    /// again" from anywhere in the app, which queues locally as in Local mode.
    pub(super) async fn hand_jobs_to_cloud(&self, cx: &mut AsyncApp) -> Result<(), Failure> {
        let mut handed = false;
        loop {
            let jobs = self
                .store(cx, |store| store.cloud_jobs(MAX_PUSH, now()))
                .await?;
            if jobs.is_empty() {
                let released = self.store(cx, Store::release_blocked_jobs).await?;
                // Only on a change: a refresh wakes the next push, which would hand over
                // an empty queue again, forever.
                if handed || released {
                    self.library.update(cx, Library::refresh_soon);
                }
                return Ok(());
            }
            for folders_only in [false, true] {
                let bookmarks: Vec<String> = jobs
                    .iter()
                    .filter(|job| job.folders_only == folders_only)
                    .map(|job| job.bookmark.clone())
                    .collect();
                if bookmarks.is_empty() {
                    continue;
                }
                self.organize_in_cloud(bookmarks, folders_only, cx).await?;
                handed = true;
            }
        }
    }

    /// A free account without room this month organizes on this Mac again, so the jobs
    /// the cloud refused go to this Mac's organizer.
    pub(super) async fn reclaim_refused_jobs(&self, cx: &mut AsyncApp) -> Result<(), Failure> {
        let reclaimed = self.store(cx, Store::reclaim_refused_jobs).await?;
        if reclaimed > 0 {
            tracing::info!(
                reclaimed,
                "organizing on this Mac what the cloud didn't take"
            );
            cx.update(Organizer::wake);
            self.library.update(cx, Library::refresh_soon);
        }
        Ok(())
    }

    async fn organize_in_cloud(
        &self,
        bookmarks: Vec<String>,
        folders_only: bool,
        cx: &mut AsyncApp,
    ) -> Result<(), Failure> {
        let request = OrganizeRequest {
            scope: OrganizeScope::Bookmarks(bookmarks.clone()),
            folders_only,
        };
        let api = self.api.clone();
        let response = net(cx, async move { api.organize(&request).await }).await?;
        let (accepted, refused) = response.split(bookmarks);
        tracing::info!(
            accepted = accepted.len(),
            refused = refused.len(),
            queued = response.queued,
            pending = response.pending,
            folders_only,
            "organizing in the cloud"
        );
        // The response can exhaust allowance before the next account pull shows it, and a
        // refusal may mean organizing moved to this Mac. Either way the next round pulls
        // the server's `queued`, since this round's pull came before the handover.
        let wake = if response.pending > 0 || !refused.is_empty() {
            Wake::Account
        } else {
            Wake::Pull
        };
        self.this
            .update(cx, |this, _| this.wake(wake))
            .map_err(|_| Failure::Stopped)?;
        self.store(cx, move |store| {
            store.hand_to_cloud(&accepted, &refused, now())
        })
        .await
    }
}
