//! Local changes that wait while sync is offline, counted on the background executor so
//! the account menu can say how many without touching the database.

use gpui::{AppContext as _, Context, Task};

use super::{Cloud, SyncStatus, Wake};

/// The last count, and the count in progress.
#[derive(Default)]
pub(super) struct Waiting {
    count: usize,
    counting: Option<Task<()>>,
}

impl Cloud {
    /// Local changes not sent yet, as of the last count. Counted only while offline, the
    /// one time the number is shown.
    pub fn waiting_changes(&self) -> usize {
        self.waiting.count
    }

    /// Syncs now instead of waiting out the retry delay.
    pub fn retry_sync(&mut self, _cx: &mut Context<'_, Cloud>) {
        self.wake(Wake::Pull);
    }

    /// Recounts after the library or the sync status changed. A newer count replaces one
    /// still running.
    pub(super) fn count_waiting(&mut self, cx: &mut Context<'_, Cloud>) {
        if self.sync != SyncStatus::Offline {
            self.waiting.counting = None;
            return;
        }
        let database = self.library.read(cx).database();
        let count = cx.background_spawn(async move { database.read().unpushed_changes() });
        self.waiting.counting = Some(cx.spawn(async move |this, cx| {
            let count = count.await;
            let _ = this.update(cx, |this, cx| match count {
                Ok(count) if count != this.waiting.count => {
                    this.waiting.count = count;
                    cx.notify();
                }
                Ok(_) => {}
                Err(err) => tracing::warn!("can't count the changes waiting to sync: {err}"),
            });
        }));
    }
}
