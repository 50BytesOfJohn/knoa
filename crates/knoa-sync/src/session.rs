//! The signed-in session: hands out a valid access token, refreshing it shortly before it
//! runs out. One refresh at a time, however many requests are waiting for it.

use std::time::{Duration, Instant};

use futures::channel::mpsc::UnboundedSender;
use tokio::sync::Mutex;

use crate::{Config, Error, Result, Tokens, oauth};

/// Refresh this long before the access token expires, so no request carries one that
/// runs out on the way.
const REFRESH_AHEAD: Duration = Duration::from_mins(5);

/// What the app has to act on: store a rotated refresh token, or show that the user has
/// been signed out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent {
    Rotated(String),
    SignedOut,
}

pub struct Session {
    http: reqwest::Client,
    config: Config,
    state: Mutex<State>,
    events: UnboundedSender<SessionEvent>,
}

struct State {
    refresh: String,
    access: Option<(String, Instant)>,
    signed_out: bool,
}

impl Session {
    /// A session from a stored refresh token, with tokens from a sign-in if there are any.
    pub fn new(
        http: reqwest::Client,
        config: Config,
        refresh: String,
        tokens: Option<Tokens>,
        events: UnboundedSender<SessionEvent>,
    ) -> Session {
        let access = tokens.map(|tokens| (tokens.access, tokens.expires_at));
        Session {
            http,
            config,
            state: Mutex::new(State {
                refresh,
                access,
                signed_out: false,
            }),
            events,
        }
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    /// A token good for at least a few minutes.
    pub async fn access_token(&self) -> Result<String> {
        let mut state = self.state.lock().await;
        if state.signed_out {
            return Err(Error::SignedOut);
        }
        if let Some((token, expires_at)) = &state.access
            && Instant::now() + REFRESH_AHEAD < *expires_at
        {
            return Ok(token.clone());
        }
        match oauth::refresh(&self.http, &self.config, &state.refresh).await {
            Ok(tokens) => {
                if let Some(refresh) = tokens.refresh
                    && refresh != state.refresh
                {
                    state.refresh.clone_from(&refresh);
                    let _ = self.events.unbounded_send(SessionEvent::Rotated(refresh));
                }
                state.access = Some((tokens.access.clone(), tokens.expires_at));
                Ok(tokens.access)
            }
            Err(Error::SignedOut) => {
                state.signed_out = true;
                state.access = None;
                let _ = self.events.unbounded_send(SessionEvent::SignedOut);
                Err(Error::SignedOut)
            }
            Err(err) => Err(err),
        }
    }

    /// The API refused `token`: the next request gets a fresh one.
    pub async fn invalidate(&self, token: &str) {
        let mut state = self.state.lock().await;
        if state.access.as_ref().is_some_and(|(t, _)| t == token) {
            state.access = None;
        }
    }

    /// Revokes the refresh token at Clerk. The session can't be used afterwards.
    pub async fn revoke(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        state.signed_out = true;
        state.access = None;
        oauth::revoke(&self.http, &self.config, &state.refresh).await
    }
}
