//! The sync loop: push the outbox, hand the queue to the cloud organizer when it does
//! the work, pull what changed, and move thumbnails. It runs after local changes, when
//! the socket pokes, and every few minutes in case a poke was missed.

mod account;
mod organize;
mod pull;
mod push;

use std::{sync::Arc, time::Duration};

use futures::{FutureExt as _, StreamExt as _, channel::mpsc::UnboundedReceiver};
use gpui::{AppContext as _, AsyncApp, Entity, WeakEntity};
use knoa_core::protocol::{Account, Organizer as Who};
use knoa_fetch::Fetcher;
use knoa_store::Store;
use knoa_sync::Api;

use super::{Cloud, SyncStatus, Wake, thumbnails::Thumbnails};
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
        if (wanted.account || self.account.is_none())
            && let Some(other) = self.refresh_account(cx).await?
        {
            return Ok(Round::OtherAccount(other));
        }
        // A push that keeps failing mustn't keep this Mac from hearing what changed elsewhere.
        let pushed = self.push_and_recheck(cx).await;
        if let Ok(Some(other)) = pushed {
            return Ok(Round::OtherAccount(other));
        }
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

    /// Pushes, then sends again what [`Engine::recheck_room`] queued, so bookmarks that
    /// fit now go in this round rather than with the next change here. Returns the
    /// account when the library is linked to another one.
    async fn push_and_recheck(&mut self, cx: &mut AsyncApp) -> Result<Option<Account>, Failure> {
        if !self.push(cx).await? {
            return Ok(None);
        }
        let other = self.recheck_room(cx).await?;
        if other.is_none() {
            self.push(cx).await?;
        }
        Ok(other)
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
