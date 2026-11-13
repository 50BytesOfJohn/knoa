//! Sending the outbox, and saying what the server kept off it.

use std::collections::HashSet;

use gpui::AsyncApp;
use knoa_core::protocol::{MAX_PUSH, MutationResult, MutationStatus, RejectReason};

use super::{Engine, Failure, net};
use crate::services::cloud::SyncStatus;

impl Engine {
    /// Sends the outbox, a batch at a time, until it is empty. True when the server
    /// answered any of it.
    pub(super) async fn push(&self, cx: &mut AsyncApp) -> Result<bool, Failure> {
        let mut at_most = MAX_PUSH;
        let mut refused = Refused::default();
        loop {
            let batch = self
                .store(cx, move |store| store.next_push_at_most(at_most))
                .await?;
            if batch.is_empty() {
                if refused.sent {
                    self.refused(&refused, cx).await?;
                }
                return Ok(refused.sent);
            }
            self.status(SyncStatus::Syncing(batch.len()), cx)?;
            let sent: HashSet<String> = batch.iter().map(|m| m.mid.clone()).collect();
            let api = self.api.clone();
            match net(cx, async move { api.push(batch).await }).await {
                Ok(response) => {
                    at_most = MAX_PUSH;
                    let answers = read_answers(&sent, response.results);
                    // Settling nothing would send the same batch again right away, forever.
                    if answers.is_empty() {
                        return Err(Failure::Unanswered);
                    }
                    // Every answer is final, whatever it says.
                    let settled = self.store(cx, move |store| store.settle(&answers)).await?;
                    refused.quota += settled.parked;
                    refused.invalid |= settled.invalid > 0;
                }
                Err(Failure::Sync(err)) if is_too_large(&err) => {
                    if let Some(smaller) = smaller_batch(sent.len()) {
                        at_most = smaller;
                        continue;
                    }
                    // Alone and still too large, the change can never get through.
                    tracing::warn!(?sent, "dropped a change too large for the server");
                    refused.too_large = true;
                    let dropped: Vec<String> = sent.into_iter().collect();
                    self.store(cx, move |store| store.acknowledge(&dropped))
                        .await?;
                }
                Err(failure) => {
                    // The batches before this one were answered, and what they kept off
                    // the server stays off it.
                    if refused.kept_any() {
                        self.refused(&refused, cx).await?;
                    }
                    return Err(failure);
                }
            }
            refused.sent = true;
        }
    }

    /// Says what the last push kept off the server, or clears that once a push gets
    /// through whole.
    async fn refused(&self, refused: &Refused, cx: &mut AsyncApp) -> Result<(), Failure> {
        let limit = self.account.as_ref().map_or(0, |a| a.limits.bookmarks);
        let unsent = self.store(cx, |store| store.unsent_bookmarks()).await?;
        let notice = refused.notice(limit, unsent);
        if let Some(notice) = &notice {
            tracing::warn!("{notice}");
        }
        self.this
            .update(cx, |this, cx| this.set_push_notice(notice, cx))
            .map_err(|_| Failure::Stopped)
    }
}

/// The answers that belong to `sent`, after logging the rejections the plan's limit
/// didn't cause.
fn read_answers(sent: &HashSet<String>, results: Vec<MutationResult>) -> Vec<MutationResult> {
    results
        .into_iter()
        .filter(|result| {
            let ours = sent.contains(&result.mid);
            if !ours {
                tracing::warn!(
                    mid = result.mid,
                    "the server answered a change it wasn't sent"
                );
            } else if result.status == MutationStatus::Rejected
                && result.reason != Some(RejectReason::Quota)
            {
                tracing::warn!(mid = result.mid, reason = ?result.reason, "push rejected");
            }
            ours
        })
        .collect()
}

fn is_too_large(err: &knoa_sync::Error) -> bool {
    matches!(err, knoa_sync::Error::Server { status: 413, .. })
}

/// How many of the `sent` changes to try next after the server found them too large
/// together. Halving finds the one too large on its own in a few tries; `None` when it is
/// already alone.
fn smaller_batch(sent: usize) -> Option<usize> {
    (sent > 1).then_some(sent / 2)
}

/// What one push kept off the server.
#[derive(Default)]
struct Refused {
    /// Some batch got an answer, so the notice says how this push went.
    sent: bool,
    /// New bookmarks past the plan's limit, parked until the account has room.
    quota: usize,
    /// Some change was too large for the server even alone.
    too_large: bool,
    /// Some change broke the server's rules, like a value past its column's limit.
    invalid: bool,
}

impl Refused {
    /// Whether this push kept anything off the server so far.
    fn kept_any(&self) -> bool {
        self.quota > 0 || self.too_large || self.invalid
    }

    /// `unsent`: the bookmarks the plan's limit keeps on this Mac, from this push or before.
    fn notice(&self, limit: u64, unsent: usize) -> Option<String> {
        let quota = (unsent > 0).then(|| {
            format!(
                "Your plan holds {limit} bookmarks, so {unsent} new bookmarks stayed on this Mac only."
            )
        });
        let other = if self.invalid {
            Some("Some changes couldn't be synced and stayed on this Mac only.")
        } else if self.too_large {
            Some("Some changes were too large to sync and stayed on this Mac only.")
        } else {
            None
        };
        match (quota, other) {
            (Some(quota), Some(other)) => Some(format!("{quota} {other}")),
            (quota, other) => quota.or(other.map(str::to_owned)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halving_a_batch_too_large_isolates_one_change() {
        let mut tries = 0;
        let mut at_most = MAX_PUSH;
        while let Some(smaller) = smaller_batch(at_most) {
            at_most = smaller;
            tries += 1;
        }
        assert_eq!(at_most, 1);
        assert!(tries <= 9, "{tries} tries");
    }
}
