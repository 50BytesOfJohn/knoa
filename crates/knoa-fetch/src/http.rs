//! The HTTP client: timeouts, size limits and the User-Agent fallback.

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

const PAGE_LIMIT: usize = 4 * 1024 * 1024;
pub const IMAGE_LIMIT: usize = 8 * 1024 * 1024;

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
        let build = |ua: &str| {
            Client::builder()
                .user_agent(ua)
                .timeout(Duration::from_secs(20))
                .connect_timeout(Duration::from_secs(8))
                .redirect(reqwest::redirect::Policy::limited(8))
                .build()
        };
        Ok(Http {
            knoa: build(KNOA_UA)?,
            browser: build(BROWSER_UA)?,
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
    let mut response = client.get(url.clone()).send().await?;
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let final_url = response.url().clone();
    let status = response.status();
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        body.extend_from_slice(&chunk);
        if body.len() > limit {
            // A huge page's start holds its metadata; the rest isn't worth the memory.
            body.truncate(limit);
            break;
        }
    }
    Ok(Response {
        url: final_url,
        status,
        content_type,
        body,
    })
}
