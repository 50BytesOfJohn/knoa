//! The socket that tells this Mac when the library changed elsewhere. It carries no data:
//! a `poke` means "pull", an `account` message means "fetch the account again".

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use futures::{SinkExt as _, StreamExt as _, channel::mpsc::UnboundedSender};
use knoa_core::protocol::{PING, PONG, SOCKET_PROTOCOL, SOCKET_TOKEN_PREFIX, ServerMessage};
use tokio_tungstenite::tungstenite::{
    Message,
    client::IntoClientRequest as _,
    http::{HeaderValue, header::SEC_WEBSOCKET_PROTOCOL},
    protocol::frame::coding::CloseCode,
};

use crate::{Api, Error, Result};

/// Cloudflare drops idle sockets after about a minute and a half.
const PING_EVERY: Duration = Duration::from_secs(30);
const MIN_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_mins(5);
/// A connection that lasted this long was healthy, so the next one starts from scratch.
const STABLE: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketEvent {
    /// Connected or reconnected: anything may have changed in between.
    Connected,
    Message(ServerMessage),
    Disconnected,
}

/// Keeps the socket connected, with a fresh token each time, until `events` is closed
/// or the session ends.
pub async fn run_socket(api: Arc<Api>, events: UnboundedSender<SocketEvent>) {
    let mut backoff = MIN_BACKOFF;
    loop {
        let started = Instant::now();
        let mut displaced = false;
        match connect_once(&api, &events).await {
            Err(Error::SignedOut) => return,
            Err(err) => tracing::debug!("cloud socket: {err}"),
            Ok(Ended::Displaced) => {
                tracing::debug!("cloud socket closed: the account has too many open");
                displaced = true;
            }
            Ok(Ended::Closed) => {}
        }
        if events.unbounded_send(SocketEvent::Disconnected).is_err() {
            return;
        }
        // Coming straight back would close another client's socket, which would come
        // back in turn; backing off instead lets the churn settle.
        if started.elapsed() > STABLE && !displaced {
            backoff = MIN_BACKOFF;
        }
        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }
}

/// How a connection that didn't fail ended.
enum Ended {
    Closed,
    /// The server keeps a few sockets per account and closed this one, the oldest, to
    /// make room for a new one.
    Displaced,
}

async fn connect_once(api: &Api, events: &UnboundedSender<SocketEvent>) -> Result<Ended> {
    let token = api.session().access_token().await?;
    let mut request = api.socket_url()?.as_str().into_client_request()?;
    let protocols = format!("{SOCKET_PROTOCOL}, {SOCKET_TOKEN_PREFIX}{token}");
    let protocols = HeaderValue::from_str(&protocols)
        .map_err(|_| Error::Config("the access token can't go in a header".into()))?;
    request
        .headers_mut()
        .insert(SEC_WEBSOCKET_PROTOCOL, protocols);
    let (stream, _) = tokio_tungstenite::connect_async(request).await?;
    tracing::debug!("cloud socket connected");
    if events.unbounded_send(SocketEvent::Connected).is_err() {
        return Ok(Ended::Closed);
    }
    let (mut write, mut read) = stream.split();
    let mut ping = tokio::time::interval(PING_EVERY);
    let mut heard_at = Instant::now();
    loop {
        tokio::select! {
            _ = ping.tick() => {
                // Two pings without an answer: the connection is gone, though nobody said so.
                if heard_at.elapsed() > PING_EVERY * 2 + Duration::from_secs(5) {
                    return Ok(Ended::Closed);
                }
                write.send(Message::text(PING)).await?;
            }
            message = read.next() => {
                heard_at = Instant::now();
                match message {
                    Some(Ok(Message::Close(Some(frame)))) if frame.code == CloseCode::Policy => {
                        return Ok(Ended::Displaced);
                    }
                    None | Some(Ok(Message::Close(_))) => return Ok(Ended::Closed),
                    Some(Err(err)) => return Err(err.into()),
                    // The app stopped listening; the caller sees that too, and stops.
                    Some(Ok(Message::Text(text))) if text.as_str() != PONG => {
                        if !forward(text.as_str(), events) {
                            return Ok(Ended::Closed);
                        }
                    }
                    Some(Ok(_)) => {}
                }
            }
        }
    }
}

/// Passes a server message on; `false` once nobody listens.
fn forward(text: &str, events: &UnboundedSender<SocketEvent>) -> bool {
    match serde_json::from_str::<ServerMessage>(text) {
        Ok(message) => events.unbounded_send(SocketEvent::Message(message)).is_ok(),
        Err(err) => {
            tracing::debug!("cloud socket: unknown message {text:?}: {err}");
            true
        }
    }
}
