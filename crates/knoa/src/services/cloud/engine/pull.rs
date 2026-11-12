//! Pulling what changed on the server.

use gpui::AsyncApp;

use super::{Engine, Failure, net};
use crate::{services::cloud::SyncStatus, state::library::Library};

impl Engine {
    /// Pulls and applies every page after the cursor, one transaction per page, and says
    /// how many changes it applied.
    pub(in crate::services::cloud) async fn pull(
        &self,
        cx: &mut AsyncApp,
    ) -> Result<usize, Failure> {
        let mut applied = 0;
        loop {
            let Some(cursor) = self.store(cx, |store| store.cursor()).await? else {
                return Ok(applied);
            };
            let api = self.api.clone();
            let page = net(cx, async move { api.pull(cursor).await }).await?;
            let more = page.more;
            if !page.changes.is_empty() {
                applied += page.changes.len();
                tracing::debug!(changes = page.changes.len(), cursor = page.cursor, "pulled");
                self.status(SyncStatus::Syncing(page.changes.len()), cx)?;
                self.store(cx, move |store| {
                    store.apply_remote(&page.changes, page.cursor)
                })
                .await?;
                self.library.update(cx, Library::refresh_soon);
            } else if page.cursor != cursor {
                self.store(cx, move |store| store.apply_remote(&[], page.cursor))
                    .await?;
            }
            if !more {
                return Ok(applied);
            }
        }
    }
}
