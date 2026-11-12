//! The sync loop: push the outbox, hand the queue to the cloud organizer when it does
//! the work, pull what changed, and move thumbnails. It runs after local changes, when
//! the socket pokes, and every few minutes in case a poke was missed.

use std::{collections::HashSet, sync::Arc, time::Duration};

use futures::{FutureExt as _, StreamExt as _, channel::mpsc::UnboundedReceiver};
use gpui::{AppContext as _, AsyncApp, Entity, WeakEntity};
use knoa_core::protocol::{
    Account, MAX_PUSH, MutationResult, MutationStatus, OrganizeRequest, OrganizeScope,
    Organizer as Who, RejectReason,
};
use knoa_fetch::Fetcher;
use knoa_store::Store;
use knoa_sync::Api;

use super::{Cloud, SyncStatus, Wake, link, thumbnails::Thumbnails};
use crate::{
    services::network::Network,
    state::{database::Database, library::Library},
};

/// A pull now and then, in case the socket missed a change.
const BACKSTOP: Duration = Duration::from_mins(5);
/// Local changes come in bursts (typing tags, an import); push them together.
const DEBOUNCE: Duration = Duration::from_secs(1);
const MIN_RETRY: Duration = Duration::from_secs(15);
const MAX_RETRY: Duration = Duration::from_mins(5);

/// Why a round stopped early.
pub(super) enum Failure {
    Sync(knoa_sync::Error),
    Store(knoa_store::Error),
    /// The server's answer to a push named none of the changes sent.
    Unanswered,
    /// The entity is gone: the account signed out.
    Stopped,
}

impl From<knoa_sync::Error> for Failure {
    fn from(err: knoa_sync::Error) -> Failure {
        Failure::Sync(err)
    }
}

impl From<knoa_store::Error> for Failure {
    fn from(err: knoa_store::Error) -> Failure {
        Failure::Store(err)
    }
}

/// What the next round has to do besides pushing, which every round does.
#[derive(Clone, Copy, Default)]
struct Wanted {
    account: bool,
    pull: bool,
}

impl Wanted {
    /// A pull also asks for the account again, which keeps usage on the Account page
    /// current and notices a plan's allowance running out.
    fn add(&mut self, wake: Wake) {
        match wake {
            Wake::Push => {}
            Wake::Pull | Wake::Account => {
                self.pull = true;
                self.account = true;
            }
        }
    }
}

/// Everything a round needs, owned so the loop can hold it across awaits.
pub(super) struct Engine {
    pub this: WeakEntity<Cloud>,
    pub api: Arc<Api>,
    pub database: Arc<Database>,
    pub library: Entity<Library>,
    pub fetcher: Arc<Fetcher>,
    account: Option<Account>,
}

pub(super) async fn run(
    this: WeakEntity<Cloud>,
    api: Arc<Api>,
    library: Entity<Library>,
    fetcher: Arc<Fetcher>,
    mut wake: UnboundedReceiver<Wake>,
    cx: &mut AsyncApp,
) {
    let database = cx.update(|cx| library.read(cx).database());
    let mut engine = Engine {
        this,
        api,
        database,
        library,
        fetcher,
        account: None,
    };
    let mut thumbnails = Thumbnails::default();
    let mut wanted = Wanted {
        account: true,
        pull: true,
    };
    let mut retry: Option<Duration> = None;
    loop {
        let outcome = engine.round(wanted, &mut thumbnails, cx).await;
        let status = match outcome {
            Ok(Round::Done) => {
                retry = None;
                wanted = Wanted::default();
                SyncStatus::UpToDate
            }
            Ok(Round::OtherAccount(account)) => {
                let _ = engine
                    .this
                    .update(cx, |this, cx| this.found_other_account(account, cx));
                return;
            }
            Err(Failure::Stopped | Failure::Sync(knoa_sync::Error::SignedOut)) => return,
            Err(failure) => {
                retry = Some(retry.map_or(MIN_RETRY, |wait| (wait * 2).min(MAX_RETRY)));
                wanted = Wanted {
                    account: engine.account.is_none(),
                    pull: true,
                };
                status_of(&failure)
            }
        };
        if engine
            .this
            .update(cx, |this, cx| this.set_sync(status, cx))
            .is_err()
        {
            return;
        }
        if !wait(&mut wake, &mut wanted, retry.unwrap_or(BACKSTOP), cx).await {
            return;
        }
    }
}

/// Sleeps until woken or `timeout`. False once the sender is gone.
async fn wait(
    wake: &mut UnboundedReceiver<Wake>,
    wanted: &mut Wanted,
    timeout: Duration,
    cx: &AsyncApp,
) -> bool {
    futures::select! {
        woken = wake.next() => match woken {
            Some(woken) => wanted.add(woken),
            None => return false,
        },
        () = cx.background_executor().timer(timeout).fuse() => wanted.add(Wake::Pull),
    }
    cx.background_executor().timer(DEBOUNCE).await;
    while let Ok(woken) = wake.try_recv() {
        wanted.add(woken);
    }
    true
}

fn status_of(failure: &Failure) -> SyncStatus {
    match failure {
        Failure::Sync(err) if err.is_transient() => {
            tracing::info!("cloud sync is offline: {err}");
            SyncStatus::Offline
        }
        Failure::Sync(err) => {
            tracing::warn!("cloud sync failed: {err}");
            SyncStatus::Error(err.to_string())
        }
        Failure::Store(err) => {
            tracing::error!("cloud sync can't use the library: {err}");
            SyncStatus::Error(err.to_string())
        }
        Failure::Unanswered => {
            tracing::warn!("cloud sync failed: the server answered none of the changes sent");
            SyncStatus::Error("The server didn't take the latest changes.".into())
        }
        Failure::Stopped => SyncStatus::Starting,
    }
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
    /// New bookmarks past the plan's limit.
    quota: usize,
    /// Some change was too large for the server even alone.
    too_large: bool,
}

impl Refused {
    fn notice(&self, limit: u64) -> Option<String> {
        let quota = (self.quota > 0).then(|| {
            format!(
                "Your plan holds {limit} bookmarks, so {} new bookmarks stayed on this Mac only.",
                self.quota
            )
        });
        let too_large = self
            .too_large
            .then(|| "Some changes were too large to sync and stayed on this Mac only.".to_owned());
        match (quota, too_large) {
            (Some(quota), Some(too_large)) => Some(format!("{quota} {too_large}")),
            (quota, too_large) => quota.or(too_large),
        }
    }
}

enum Round {
    Done,
    /// The library is linked to another account than this one.
    OtherAccount(Account),
}

impl Engine {
    async fn round(
        &mut self,
        wanted: Wanted,
        thumbnails: &mut Thumbnails,
        cx: &mut AsyncApp,
    ) -> Result<Round, Failure> {
        if wanted.account || self.account.is_none() {
            let account = self.fetch_account(cx).await?;
            self.this
                .update(cx, |this, cx| this.set_account(account.clone(), cx))
                .map_err(|_| Failure::Stopped)?;
            if link::ensure(self, &account, cx).await? == link::Linked::OtherAccount {
                return Ok(Round::OtherAccount(account));
            }
            self.account = Some(account);
        }
        // A push that keeps failing mustn't keep this Mac from hearing what changed elsewhere.
        let pushed = self.push(cx).await;
        let organizer = self.account.as_ref().map(|a| a.organizer);
        if pushed.is_ok() && organizer == Some(Who::Cloud) {
            self.hand_jobs_to_cloud(cx).await?;
        }
        if wanted.pull {
            self.pull(cx).await?;
        }
        pushed?;
        thumbnails.download(self, cx).await?;
        if organizer == Some(Who::Device) {
            thumbnails.upload(self, cx).await?;
        }
        Ok(Round::Done)
    }

    /// `/v1/me`, with the email from Clerk when the API doesn't know it yet. Asked once
    /// per connection, though the account is fetched on every pull.
    async fn fetch_account(&self, cx: &AsyncApp) -> Result<Account, Failure> {
        let api = self.api.clone();
        let known_email = self.account.as_ref().and_then(|a| a.email.clone());
        net(cx, async move {
            let mut account = api.me().await?;
            account.email = account.email.or(known_email);
            if account.email.is_none() {
                let session = api.session();
                let token = session.access_token().await?;
                account.email = knoa_sync::user_email(session.http(), session.config(), &token)
                    .await
                    .unwrap_or_default();
            }
            Ok(account)
        })
        .await
    }

    /// Sends the outbox, a batch at a time, until it is empty.
    async fn push(&self, cx: &mut AsyncApp) -> Result<(), Failure> {
        let mut at_most = MAX_PUSH;
        let mut refused = Refused::default();
        loop {
            let batch = self
                .store(cx, move |store| store.next_push_at_most(at_most))
                .await?;
            if batch.is_empty() {
                return if refused.sent {
                    self.refused(&refused, cx)
                } else {
                    Ok(())
                };
            }
            self.status(SyncStatus::Syncing(batch.len()), cx)?;
            let sent: HashSet<String> = batch.iter().map(|m| m.mid.clone()).collect();
            let api = self.api.clone();
            let answered = match net(cx, async move { api.push(batch).await }).await {
                Ok(response) => {
                    at_most = MAX_PUSH;
                    self.read_answers(&sent, response.results, &mut refused)
                }
                Err(Failure::Sync(err)) if is_too_large(&err) => {
                    if let Some(smaller) = smaller_batch(sent.len()) {
                        at_most = smaller;
                        continue;
                    }
                    // Alone and still too large, the change can never get through.
                    tracing::warn!(?sent, "dropped a change too large for the server");
                    refused.too_large = true;
                    sent.into_iter().collect()
                }
                Err(failure) => return Err(failure),
            };
            refused.sent = true;
            // Acknowledging nothing would send the same batch again right away, forever.
            if answered.is_empty() {
                return Err(Failure::Unanswered);
            }
            // Every answer is final, whatever it says.
            self.store(cx, move |store| store.acknowledge(&answered))
                .await?;
        }
    }

    /// The ids of the answers that belong to `sent`, after logging the rejections and
    /// counting the ones the plan's limit refused.
    fn read_answers(
        &self,
        sent: &HashSet<String>,
        results: Vec<MutationResult>,
        refused: &mut Refused,
    ) -> Vec<String> {
        let mut answered = Vec::with_capacity(results.len());
        for result in results {
            if !sent.contains(&result.mid) {
                tracing::warn!(
                    mid = result.mid,
                    "the server answered a change it wasn't sent"
                );
                continue;
            }
            if result.status == MutationStatus::Rejected {
                match result.reason {
                    Some(RejectReason::Quota) => refused.quota += 1,
                    reason => tracing::warn!(mid = result.mid, ?reason, "push rejected"),
                }
            }
            answered.push(result.mid);
        }
        answered
    }

    /// Says what the last push kept off the server, or clears that once a push gets
    /// through whole.
    fn refused(&self, refused: &Refused, cx: &mut AsyncApp) -> Result<(), Failure> {
        let limit = self.account.as_ref().map_or(0, |a| a.limits.bookmarks);
        let notice = refused.notice(limit);
        if let Some(notice) = &notice {
            tracing::warn!("{notice}");
        }
        self.this
            .update(cx, |this, cx| this.set_push_notice(notice, cx))
            .map_err(|_| Failure::Stopped)
    }

    /// Asks the cloud organizer to take this Mac's queue: new bookmarks, and "Organize
    /// again" from anywhere in the app, which queues locally as in Local mode.
    async fn hand_jobs_to_cloud(&self, cx: &mut AsyncApp) -> Result<(), Failure> {
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

    /// Pulls and applies every page after the cursor, one transaction per page.
    async fn pull(&self, cx: &mut AsyncApp) -> Result<(), Failure> {
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

    pub(super) fn status(&self, status: SyncStatus, cx: &mut AsyncApp) -> Result<(), Failure> {
        self.this
            .update(cx, |this, cx| this.set_sync(status, cx))
            .map_err(|_| Failure::Stopped)
    }

    /// Runs `work` against the store on the background executor.
    pub(super) async fn store<T: Send + 'static>(
        &self,
        cx: &AsyncApp,
        work: impl FnOnce(&mut Store) -> knoa_store::Result<T> + Send + 'static,
    ) -> Result<T, Failure> {
        let database = self.database.clone();
        Ok(cx
            .background_spawn(async move { work(&mut database.write()) })
            .await?)
    }
}

/// Runs `work` on the network runtime.
pub(super) async fn net<T: Send + 'static>(
    cx: &AsyncApp,
    work: impl Future<Output = knoa_sync::Result<T>> + Send + 'static,
) -> Result<T, Failure> {
    let handle = cx.update(|cx| Network::spawn(cx, work));
    match handle.await {
        Ok(result) => Ok(result?),
        Err(err) => Err(Failure::Sync(knoa_sync::Error::Io(std::io::Error::other(
            err.to_string(),
        )))),
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
