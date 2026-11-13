//! Cloud mode: signing in to a Knoa account, keeping the library in sync with it, and
//! handing organizing to the cloud when the account has Pro.
//!
//! Network work runs on tokio and database work on the background executor; the sync
//! loop in `engine` only coordinates, like the organizer's. Signed out, or in a build
//! without a cloud, none of this runs and the app is Local mode as always.

mod config;
mod connection;
mod engine;
mod link;
mod replacement;
mod thumbnails;

use std::sync::Arc;

use futures::channel::mpsc::UnboundedSender;
use gpui::{App, AppContext as _, Context, Entity, Global, Subscription, Task};
use knoa_core::protocol::{Account, Organizer as Who};
use knoa_fetch::Fetcher;
use knoa_sync::Api;

pub use self::config::CloudConfig;
use crate::{
    services::{generation::Generation, organizer::Organizer},
    state::{
        library::{Library, LibraryChanged},
        settings::SettingsFile,
    },
};

pub struct Cloud {
    /// `None` in a build without a cloud, or one whose cloud setup is broken.
    backend: Option<Backend>,
    /// Why Cloud mode couldn't start, when the build has a cloud.
    unavailable: Option<String>,
    library: Entity<Library>,
    fetcher: Arc<Fetcher>,
    phase: Phase,
    generation: Generation,
    sync: SyncStatus,
    last_synced: Option<jiff::Timestamp>,
    /// What the last push kept off the server: bookmarks past the plan's limit, or
    /// changes too large to send.
    push_notice: Option<String>,
    /// Why the last sign-in didn't work, until the next one starts.
    problem: Option<String>,
    /// Stores refresh tokens Clerk rotates, for as long as the session lasts.
    session_events: Option<Task<()>>,
    _library_changes: Subscription,
}

/// The cloud this build talks to, and the client that reaches it.
#[derive(Clone)]
struct Backend {
    config: CloudConfig,
    http: reqwest::Client,
}

impl Backend {
    /// `None` when this build has no cloud to sign in to.
    fn load() -> anyhow::Result<Option<Backend>> {
        let Some(config) = CloudConfig::load()? else {
            return Ok(None);
        };
        let http = knoa_sync::http_client()?;
        Ok(Some(Backend { config, http }))
    }
}

/// Where signing in stands.
pub enum Phase {
    SignedOut,
    /// Waiting for the browser. Dropping the task stops listening for it.
    SigningIn(#[expect(dead_code, reason = "held so the sign-in runs until dropped")] Task<()>),
    /// Signed in, but this library is linked to another account. Nothing syncs until the
    /// user decides.
    OtherAccount(Connection),
    SignedIn(Connection),
}

/// A signed-in account and the work that keeps it in sync. Dropping it stops all of it.
pub struct Connection {
    api: Arc<Api>,
    account: Option<Account>,
    wake: UnboundedSender<Wake>,
    socket: tokio::task::AbortHandle,
    _tasks: Vec<Task<()>>,
    replacement: Option<Task<()>>,
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.socket.abort();
    }
}

/// What woke the sync loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Wake {
    /// Something changed in the library here.
    Push,
    /// Something changed on the server.
    Pull,
    /// The account changed, like a new subscription.
    Account,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncStatus {
    Starting,
    UpToDate,
    Syncing(usize),
    Offline,
    Error(String),
}

const ALLOWANCE_USED_UP: &str = "This month's organizing allowance is used up. Knoa keeps your organizing requests and resumes them automatically when allowance becomes available.";

struct CloudGlobal(Entity<Cloud>);

impl Global for CloudGlobal {}

impl Cloud {
    /// Cloud mode is optional: a broken cloud setup leaves it unavailable, with the
    /// reason on the Account page, and the app starts as in Local mode.
    pub fn init(library: Entity<Library>, fetcher: Arc<Fetcher>, cx: &mut App) {
        let (backend, unavailable) = match Backend::load() {
            Ok(backend) => (backend, None),
            Err(err) => {
                tracing::error!("cloud mode can't start: {err:#}");
                (None, Some(err.to_string()))
            }
        };
        if let Some(Backend { config, .. }) = &backend {
            tracing::info!(
                api = config.sync.api.origin().ascii_serialization(),
                issuer = %config.sync.issuer,
                "cloud mode is available"
            );
        } else if unavailable.is_none() {
            tracing::info!("cloud mode isn't available in this build");
        }
        let cloud = cx.new(|cx| {
            let changes = cx.subscribe(&library, |this: &mut Cloud, _, _: &LibraryChanged, _| {
                this.wake(Wake::Push);
            });
            Cloud {
                backend,
                unavailable,
                library,
                fetcher,
                phase: Phase::SignedOut,
                generation: Generation::default(),
                sync: SyncStatus::Starting,
                last_synced: None,
                push_notice: None,
                problem: None,
                session_events: None,
                _library_changes: changes,
            }
        });
        cx.set_global(CloudGlobal(cloud.clone()));
        cloud.update(cx, Cloud::restore);
    }

    pub fn entity(cx: &App) -> Entity<Cloud> {
        cx.global::<CloudGlobal>().0.clone()
    }

    /// `None` in a build that has no cloud to sign in to, or when it couldn't start.
    pub fn config(&self) -> Option<&CloudConfig> {
        self.backend.as_ref().map(|backend| &backend.config)
    }

    /// Why Cloud mode couldn't start in a build that has it.
    pub fn unavailable(&self) -> Option<&str> {
        self.unavailable.as_deref()
    }

    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    /// The signed-in account, once the server has said who it is.
    pub fn account(&self) -> Option<&Account> {
        match &self.phase {
            Phase::SignedIn(connection) | Phase::OtherAccount(connection) => {
                connection.account.as_ref()
            }
            Phase::SignedOut | Phase::SigningIn(_) => None,
        }
    }

    pub fn sync(&self) -> &SyncStatus {
        &self.sync
    }

    pub fn last_synced(&self) -> Option<jiff::Timestamp> {
        self.last_synced
    }

    /// Something the user should know that isn't an error of the sync itself: the plan's
    /// limits holding back changes or organizing, or changes too large to send.
    pub fn notice(&self) -> Option<&str> {
        self.push_notice
            .as_deref()
            .or_else(|| self.allowance_used_up().then_some(ALLOWANCE_USED_UP))
    }

    /// Read from the account rather than remembered, so it goes away by itself with the
    /// next month or an upgrade.
    fn allowance_used_up(&self) -> bool {
        self.account().is_some_and(|account| {
            account.organizer == Who::Cloud
                && account.usage.organized_this_month >= account.limits.organized_per_month
        })
    }

    pub fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }

    fn wake(&self, wake: Wake) {
        if let Phase::SignedIn(connection) = &self.phase {
            let _ = connection.wake.unbounded_send(wake);
        }
    }

    /// The server said who the account is and what it has. Asked on every pull, so this
    /// only writes and redraws when something changed.
    fn set_account(&mut self, account: Account, cx: &mut Context<'_, Cloud>) {
        match account.organizer {
            Who::Cloud => set_cloud_organizes(true, "the account is organized in the cloud", cx),
            Who::Device => set_cloud_organizes(false, "the account is organized on this Mac", cx),
        }
        if SettingsFile::get(cx).account.as_ref() != Some(&account) {
            SettingsFile::update(cx, |s| s.account = Some(account.clone()));
        }
        if let Phase::SignedIn(connection) | Phase::OtherAccount(connection) = &mut self.phase
            && connection.account.as_ref() != Some(&account)
        {
            connection.account = Some(account);
            cx.notify();
        }
    }

    fn set_sync(&mut self, sync: SyncStatus, cx: &mut Context<'_, Cloud>) {
        if sync == SyncStatus::UpToDate {
            self.last_synced = Some(jiff::Timestamp::now());
        }
        if self.sync != sync || sync == SyncStatus::UpToDate {
            self.sync = sync;
            cx.notify();
        }
    }

    fn set_push_notice(&mut self, notice: Option<String>, cx: &mut Context<'_, Cloud>) {
        if self.push_notice != notice {
            self.push_notice = notice;
            cx.notify();
        }
    }
}

fn set_cloud_organizes(cloud: bool, why: &'static str, cx: &mut App) {
    Organizer::entity(cx).update(cx, |organizer, cx| organizer.set_cloud(cloud, why, cx));
}
