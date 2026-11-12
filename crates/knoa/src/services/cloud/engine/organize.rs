//! Handing this Mac's organizing queue to the cloud organizer, when the account has it.

use gpui::AsyncApp;
use knoa_core::protocol::{MAX_PUSH, OrganizeRequest, OrganizeScope};

use super::{Engine, Failure, net};

impl Engine {
    /// Asks the cloud organizer to take this Mac's queue: new bookmarks, and "Organize
    /// again" from anywhere in the app, which queues locally as in Local mode.
    pub(super) async fn hand_jobs_to_cloud(&self, cx: &mut AsyncApp) -> Result<(), Failure> {
        loop {
            let jobs = self.store(cx, |store| store.cloud_jobs(MAX_PUSH)).await?;
            if jobs.is_empty() {
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
            }
        }
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
        tracing::info!(
            asked = bookmarks.len(),
            queued = response.queued,
            folders_only,
            "organizing in the cloud"
        );
        self.store(cx, move |store| store.hand_to_cloud(&bookmarks))
            .await
    }
}
