//! The client on macOS: reqwest's own connection pool, with redirects checked hop by hop.

use std::time::Duration;

use reqwest::{Client, redirect};

use crate::access;

pub fn client(user_agent: &str) -> reqwest::Result<Client> {
    Client::builder()
        .user_agent(user_agent)
        .connect_timeout(Duration::from_secs(8))
        .redirect(redirect_policy())
        .build()
}

pub fn is_connect(err: &reqwest::Error) -> bool {
    err.is_connect()
}

/// Reads at most `limit` bytes: a huge page's start holds its metadata, and the rest
/// isn't worth the memory.
pub async fn read_body(mut response: reqwest::Response, limit: usize) -> reqwest::Result<Vec<u8>> {
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        body.extend_from_slice(&chunk);
        if body.len() > limit {
            body.truncate(limit);
            break;
        }
    }
    Ok(body)
}

/// Only the bookmarked URL is checked before fetching, so a public page could otherwise
/// redirect Knoa to a private host and have that page stored and sent to the AI. Stopping
/// hands back the redirect itself, which reads as an unreadable page.
fn redirect_policy() -> redirect::Policy {
    let limited = redirect::Policy::limited(8);
    redirect::Policy::custom(move |attempt| {
        if access::is_private(attempt.url()) {
            attempt.stop()
        } else {
            limited.redirect(attempt)
        }
    })
}
