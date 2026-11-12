//! The client in a Cloudflare Worker, where reqwest wraps the runtime's `fetch`.
//!
//! `fetch` follows redirects itself and offers no per-hop check. That is safe there: a
//! Worker can't reach private networks, so the redirect check that guards the desktop
//! has nothing to stop.

use futures::StreamExt as _;
use reqwest::Client;

pub fn client(user_agent: &str) -> reqwest::Result<Client> {
    Client::builder().user_agent(user_agent).build()
}

/// `fetch` reports no separate connect failures; they arrive as request errors.
pub fn is_connect(_err: &reqwest::Error) -> bool {
    false
}

/// Reads at most `limit` bytes: a huge page's start holds its metadata, and a Worker has
/// little memory to spare.
pub async fn read_body(response: reqwest::Response, limit: usize) -> reqwest::Result<Vec<u8>> {
    let mut body = Vec::new();
    let mut chunks = response.bytes_stream();
    while let Some(chunk) = chunks.next().await {
        body.extend_from_slice(&chunk?);
        if body.len() > limit {
            body.truncate(limit);
            break;
        }
    }
    Ok(body)
}
