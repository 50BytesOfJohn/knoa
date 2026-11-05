//! AI usage: storing what the provider's requests used, and reading it back for settings.

use gpui::{App, AppContext as _, BackgroundExecutor, Task};
use jiff::{ToSpan as _, Zoned, civil::Date};
use knoa_ai::Provider;
use knoa_core::ModelUsage;

use crate::state::library::{SharedStore, lock};

/// The recent window settings shows next to all time.
pub const RECENT_DAYS: i32 = 30;

/// Each model's usage over the recent window and ever, most expensive first.
#[derive(Debug, Clone, Default)]
pub struct UsageReport {
    pub recent: Vec<ModelUsage>,
    pub all_time: Vec<ModelUsage>,
}

impl UsageReport {
    pub fn is_empty(&self) -> bool {
        self.all_time.is_empty()
    }
}

/// Stores what `provider`'s requests used since this last ran. The provider is shared by
/// concurrent jobs, so this may also store requests of jobs still running; each request
/// is stored once either way.
pub async fn save(provider: &Provider, store: SharedStore, background: &BackgroundExecutor) {
    let usage = provider.take_usage();
    if usage.is_empty() {
        return;
    }
    let (provider, day) = (provider.kind().id(), today().to_string());
    let saved = background
        .spawn(async move { lock(&store).record_usage(&day, provider, &usage) })
        .await;
    if let Err(err) = saved {
        tracing::error!("can't save AI usage: {err}");
    }
}

pub fn load(store: SharedStore, cx: &App) -> Task<knoa_store::Result<UsageReport>> {
    let since = today()
        .checked_sub((RECENT_DAYS - 1).days())
        .map(|d| d.to_string())
        .ok();
    cx.background_spawn(async move {
        let store = lock(&store);
        Ok(UsageReport {
            recent: store.usage_by_model(since.as_deref())?,
            all_time: store.usage_by_model(None)?,
        })
    })
}

/// Days follow the user's calendar, not UTC.
fn today() -> Date {
    Zoned::now().date()
}
