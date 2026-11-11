//! The HTTP client: timeouts, redirects, size limits and the User-Agent fallback.

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
use native as platform;
#[cfg(target_arch = "wasm32")]
use wasm as platform;

use std::time::Duration;

use reqwest::{Client, StatusCode, header};
use serde::de::DeserializeOwned;
use url::Url;

use crate::Error;

/// An honest User-Agent works best: Hacker News answers 419 and Cursor 403 to a browser UA.
const KNOA_UA: &str = concat!(
    "Knoa/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/50BytesOfJohn/knoa)"
);
/// For sites that reject unknown agents.
const BROWSER_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0 Safari/537.36";

const TIMEOUT: Duration = Duration::from_secs(20);
const PAGE_LIMIT: usize = 4 * 1024 * 1024;
pub const IMAGE_LIMIT: usize = 8 * 1024 * 1024;

pub use platform::is_connect;

pub struct Http {
    knoa: Client,
    browser: Client,
}

pub struct Response {
    pub url: Url,
    pub status: StatusCode,
    pub content_type: String,
    pub body: Vec<u8>,
}

impl Http {
    pub fn new() -> Result<Http, Error> {
        Ok(Http {
            knoa: platform::client(KNOA_UA)?,
            browser: platform::client(BROWSER_UA)?,
        })
    }

    /// Fetches a page, retrying once with a browser User-Agent when the site refuses Knoa's.
    pub async fn page(&self, url: &Url) -> Result<Response, Error> {
        let first = get(&self.knoa, url, PAGE_LIMIT).await?;
        if matches!(first.status.as_u16(), 403 | 406 | 419 | 429 | 503)
            && let Ok(second) = get(&self.browser, url, PAGE_LIMIT).await
            && second.status.is_success()
        {
            return Ok(second);
        }
        Ok(first)
    }

    pub async fn bytes(&self, url: &Url, limit: usize) -> Result<Response, Error> {
        get(&self.knoa, url, limit).await
    }

    /// `None` for a non-success status, which for site APIs means "not something we know".
    pub async fn get_json<T: DeserializeOwned>(&self, url: &Url) -> Result<Option<T>, Error> {
        let response = get(&self.knoa, url, PAGE_LIMIT).await?;
        if !response.status.is_success() {
            return Ok(None);
        }
        Ok(serde_json::from_slice(&response.body).ok())
    }
}

async fn get(client: &Client, url: &Url, limit: usize) -> Result<Response, Error> {
    let response = client.get(url.clone()).timeout(TIMEOUT).send().await?;
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let final_url = response.url().clone();
    let status = response.status();
    let body = platform::read_body(response, limit).await?;
    Ok(Response {
        url: final_url,
        status,
        content_type,
        body,
    })
}
