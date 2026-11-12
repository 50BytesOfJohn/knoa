//! Pulling what changed on the server.

use gpui::AsyncApp;

use super::{Engine, Failure, net};
use crate::{services::cloud::SyncStatus, state::library::Library};

impl Engine {
    /// Pulls and applies every page after the cursor, one transaction per page.
    pub(in crate::services::cloud) async fn pull(&self, cx: &mut AsyncApp) -> Result<(), Failure> {
        loop {
            let Some(cursor) = self.store(cx, |store| store.cursor()).await? else {
                return Ok(());
            };
            let api = self.api.clone();
            let page = net(cx, async move { api.pull(cursor).await }).await?;
            let more = page.more;
            if !page.changes.is_empty() {
                tracing::debug!(changes = page.changes.len(), "pulled");
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
                return Ok(());
            }
        }
    }
}
