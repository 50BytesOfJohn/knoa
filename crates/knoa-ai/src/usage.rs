//! Holding what each request used until the app stores it.

use std::sync::{Mutex, MutexGuard, PoisonError};

use knoa_core::Usage;

/// Counts every request that got an answer, including answers that turned out unusable:
/// the provider bills those too.
#[derive(Debug, Default)]
pub(crate) struct Meter {
    requests: Mutex<Vec<Usage>>,
}

impl Meter {
    pub(crate) fn record(&self, usage: Usage) {
        self.lock().push(usage);
    }

    /// The requests recorded since the last call.
    pub(crate) fn take(&self) -> Vec<Usage> {
        std::mem::take(&mut *self.lock())
    }

    // A panic mid-push leaves the list valid, so a poisoned lock is still usable.
    fn lock(&self) -> MutexGuard<'_, Vec<Usage>> {
        self.requests.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
