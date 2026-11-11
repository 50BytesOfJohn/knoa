//! Signing in and out, and starting the work a signed-in account needs: the sync loop,
//! the socket, and storing tokens Clerk rotates.

use std::sync::Arc;

use futures::{StreamExt as _, channel::mpsc};
use gpui::{AppContext as _, AsyncApp, Context, TaskExt as _, WeakEntity};
use knoa_sync::{Api, Session, SessionEvent, SignIn, SocketEvent, Tokens};

use super::{Cloud, Connection, Phase, SyncStatus, Wake, engine, set_cloud_organizes};
use crate::services::{credentials::Credentials, network::Network};

impl Cloud {
    /// At launch: signed in if a refresh token is stored. Until the server answers, the
    /// account last seen decides who organizes, so the local queue doesn't start on work
    /// the cloud does.
    pub(super) fn restore(&mut self, cx: &mut Context<'_, Cloud>) {
        let Some(config) = self.config.clone() else {
            return;
        };
        let cached = crate::state::settings::SettingsFile::get(cx)
            .account
            .clone();
        if let Some(account) = &cached {
            set_cloud_organizes(
                account.organizer == knoa_core::protocol::Organizer::Cloud,
                cx,
            );
        }
        let read = Credentials::read(config.credential_url(), cx);
        cx.spawn(async move |this, cx| {
            let read = read.await;
            let _ = this.update(cx, |this, cx| match read {
                Ok(Some(token)) => this.connect(token, None, cached, cx),
                Ok(None) => {
                    this.forget_account(cx);
                    if scripted_sign_in() {
                        this.sign_in(cx);
                    }
                }
                // Signed out for this launch only: a Keychain that didn't answer is no
                // reason to delete the sign-in it holds.
                Err(err) => {
                    tracing::error!("can't read the cloud sign-in: {err:#}");
                    set_cloud_organizes(false, cx);
                }
            });
        })
        .detach();
    }

    /// Opens the browser to sign in, and finishes signing in when it comes back.
    pub fn sign_in(&mut self, cx: &mut Context<'_, Cloud>) {
        let Some(config) = self.config.clone() else {
            return;
        };
        if !matches!(self.phase, Phase::SignedOut) {
            return;
        }
        self.problem = None;
        let http = self.http.clone();
        let start = Network::spawn(cx, async move { SignIn::start(&config.sync).await });
        let task = cx.spawn(async move |this, cx| {
            let tokens = match start.await {
                Ok(Ok(sign_in)) => wait_for_browser(sign_in, http, cx).await,
                Ok(Err(err)) => Err(err.to_string()),
                Err(err) => Err(err.to_string()),
            };
            let _ = this.update(cx, |this, cx| this.signed_in(tokens, cx));
        });
        self.phase = Phase::SigningIn(task);
        cx.notify();
    }

    pub fn cancel_sign_in(&mut self, cx: &mut Context<'_, Cloud>) {
        if matches!(self.phase, Phase::SigningIn(_)) {
            self.phase = Phase::SignedOut;
            cx.notify();
        }
    }

    /// Signs out here and at Clerk. The library stays linked to the account, so signing
    /// in to the same account again carries on where it left off.
    pub fn sign_out(&mut self, cx: &mut Context<'_, Cloud>) {
        let phase = std::mem::replace(&mut self.phase, Phase::SignedOut);
        if let Phase::SignedIn(connection) | Phase::OtherAccount(connection) = phase {
            let session = connection.api.session().clone();
            Network::spawn(cx, async move {
                if let Err(err) = session.revoke().await {
                    tracing::warn!("couldn't revoke the cloud sign-in: {err}");
                }
            });
        }
        self.forget_account(cx);
        cx.notify();
    }

    /// The user agreed to replace the other account's link: this library becomes the
    /// signed-in account's, merged with what it already has.
    pub fn replace_other_account(&mut self, cx: &mut Context<'_, Cloud>) {
        let Phase::OtherAccount(connection) = &self.phase else {
            return;
        };
        let api = connection.api.clone();
        let account = connection.account.clone();
        let database = self.library.read(cx).database();
        cx.spawn(async move |this, cx| {
            let unlinked = cx
                .background_spawn(async move { database.write().unlink() })
                .await;
            let _ = this.update(cx, |this, cx| match unlinked {
                Ok(()) => this.start(api, account, cx),
                Err(err) => this.problem = Some(format!("Couldn't unlink the library: {err}")),
            });
        })
        .detach();
    }

    /// The sync loop found the library linked to another account than `account`, which
    /// the Account page names while the user decides.
    pub(super) fn found_other_account(
        &mut self,
        account: knoa_core::protocol::Account,
        cx: &mut Context<'_, Cloud>,
    ) {
        let phase = std::mem::replace(&mut self.phase, Phase::SignedOut);
        self.phase = match phase {
            Phase::SignedIn(mut connection) => {
                connection.account = Some(account);
                Phase::OtherAccount(connection)
            }
            other => other,
        };
        set_cloud_organizes(false, cx);
        cx.notify();
    }

    fn signed_in(&mut self, tokens: Result<Tokens, String>, cx: &mut Context<'_, Cloud>) {
        if !matches!(self.phase, Phase::SigningIn(_)) {
            return;
        }
        self.phase = Phase::SignedOut;
        let Some(config) = self.config.clone() else {
            return;
        };
        match tokens {
            Ok(tokens) => {
                let Some(refresh) = tokens.refresh.clone() else {
                    return;
                };
                tracing::info!("signed in to the cloud");
                Credentials::write(config.credential_url(), &refresh, cx).detach_and_log_err(cx);
                self.connect(refresh, Some(tokens), None, cx);
            }
            Err(problem) => {
                tracing::warn!("signing in failed: {problem}");
                self.problem = Some(problem);
            }
        }
        cx.notify();
    }

    /// Starts a session from a refresh token, with the tokens a sign-in just got.
    fn connect(
        &mut self,
        refresh: String,
        tokens: Option<Tokens>,
        account: Option<knoa_core::protocol::Account>,
        cx: &mut Context<'_, Cloud>,
    ) {
        let Some(config) = self.config.clone() else {
            return;
        };
        let (events, events_rx) = mpsc::unbounded();
        let session = Session::new(self.http.clone(), config.sync, refresh, tokens, events);
        let api = Arc::new(Api::new(Arc::new(session)));
        self.session_events = Some(cx.spawn(async move |this, cx| {
            on_session_events(this, events_rx, cx).await;
        }));
        self.start(api, account, cx);
    }

    /// Starts syncing as `api`'s account.
    fn start(
        &mut self,
        api: Arc<Api>,
        account: Option<knoa_core::protocol::Account>,
        cx: &mut Context<'_, Cloud>,
    ) {
        let (wake, wake_rx) = mpsc::unbounded();
        let (socket_events, socket_rx) = mpsc::unbounded();
        let socket = Network::spawn(cx, knoa_sync::run_socket(api.clone(), socket_events));
        let library = self.library.clone();
        let fetcher = self.fetcher.clone();
        let engine_api = api.clone();
        let engine = cx.spawn(async move |this, cx| {
            engine::run(this, engine_api, library, fetcher, wake_rx, cx).await;
        });
        let forward_wake = wake.clone();
        let socket_task = cx.spawn(async move |_, _| forward_socket(socket_rx, forward_wake).await);
        self.sync = SyncStatus::Starting;
        self.phase = Phase::SignedIn(Connection {
            api,
            account,
            wake,
            socket: socket.abort_handle(),
            _tasks: vec![engine, socket_task],
        });
        cx.notify();
    }

    /// Forgets the token and the account. The library and its link stay.
    fn forget_account(&mut self, cx: &mut Context<'_, Cloud>) {
        if let Some(config) = &self.config {
            Credentials::delete(config.credential_url(), cx).detach_and_log_err(cx);
        }
        crate::state::settings::SettingsFile::update(cx, |s| s.account = None);
        set_cloud_organizes(false, cx);
        self.session_events = None;
        self.sync = SyncStatus::Starting;
        self.last_synced = None;
        self.quota_notice = None;
    }
}

async fn wait_for_browser(
    sign_in: SignIn,
    http: reqwest::Client,
    cx: &mut AsyncApp,
) -> Result<Tokens, String> {
    let url = sign_in.authorize_url().to_string();
    // Lets a debug build be signed in from a browser other than the default one.
    if cfg!(debug_assertions) {
        tracing::info!("sign-in page: {url}");
    }
    if !scripted_sign_in() {
        cx.update(|cx| cx.open_url(&url));
    }
    let finish = cx.update(|cx| Network::spawn(cx, async move { sign_in.finish(&http).await }));
    // Dropped with the sign-in task on Cancel: stop listening for the browser too.
    let abort = AbortOnDrop(finish.abort_handle());
    let tokens = match finish.await {
        Ok(result) => result.map_err(|err| err.to_string()),
        Err(err) => Err(err.to_string()),
    };
    drop(abort);
    tokens
}

/// Debug builds only: `KNOA_SCRIPTED_SIGN_IN=1` starts signing in at launch and leaves
/// opening the logged page to whoever runs the test, so sign-in can be checked end to end
/// without clicking through the app or the default browser.
fn scripted_sign_in() -> bool {
    cfg!(debug_assertions) && std::env::var_os("KNOA_SCRIPTED_SIGN_IN").is_some()
}

struct AbortOnDrop(tokio::task::AbortHandle);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

async fn on_session_events(
    this: WeakEntity<Cloud>,
    mut events: mpsc::UnboundedReceiver<SessionEvent>,
    cx: &mut AsyncApp,
) {
    while let Some(event) = events.next().await {
        let _ = this.update(cx, |this, cx| match event {
            SessionEvent::Rotated(refresh) => {
                if let Some(config) = &this.config {
                    Credentials::write(config.credential_url(), &refresh, cx)
                        .detach_and_log_err(cx);
                }
            }
            SessionEvent::SignedOut => {
                tracing::warn!("the cloud sign-in was refused; signed out");
                this.phase = Phase::SignedOut;
                this.forget_account(cx);
                this.problem = Some("You were signed out. Sign in again to keep syncing.".into());
                cx.notify();
            }
        });
    }
}

async fn forward_socket(
    mut events: mpsc::UnboundedReceiver<SocketEvent>,
    wake: mpsc::UnboundedSender<Wake>,
) {
    while let Some(event) = events.next().await {
        let wake_for = match event {
            SocketEvent::Connected
            | SocketEvent::Message(knoa_core::protocol::ServerMessage::Poke { .. }) => Wake::Pull,
            SocketEvent::Message(knoa_core::protocol::ServerMessage::Account) => Wake::Account,
            SocketEvent::Disconnected => continue,
        };
        if wake.unbounded_send(wake_for).is_err() {
            return;
        }
    }
}
