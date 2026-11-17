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

use crate::{Error, access};

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
/// A preview image: larger ones are print originals, not previews.
const IMAGE_LIMIT: usize = 8 * 1024 * 1024;
/// A link to an image file is the bookmark itself, and wallpapers and photos run large.
/// Kept within what the cloud organizer's 128 MB Worker can hold next to a decode.
pub(crate) const RAW_IMAGE_LIMIT: usize = 24 * 1024 * 1024;

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
    /// The body was longer than the limit and holds only its start: fine for a page's
    /// metadata, useless for an image.
    pub truncated: bool,
}

/// How much of a body to read.
#[derive(Debug, Clone, Copy)]
enum Limit {
    /// A bookmarked URL: a page, or an image file linked directly.
    Page,
    /// A preview image.
    Image,
}

impl Limit {
    fn bytes(self, content_type: &str) -> usize {
        match self {
            Limit::Page if is_image(content_type) => RAW_IMAGE_LIMIT,
            Limit::Page => PAGE_LIMIT,
            Limit::Image => IMAGE_LIMIT,
        }
    }

    /// Whether a body over the limit is worthless. A page's start still holds its title,
    /// description and structured data; an image cut off is refused anyway.
    fn whole_or_nothing(self, content_type: &str) -> bool {
        match self {
            Limit::Page => is_image(content_type),
            Limit::Image => true,
        }
    }
}

fn is_image(content_type: &str) -> bool {
    content_type.starts_with("image/")
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
        let first = get(&self.knoa, url, Limit::Page).await?;
        if matches!(first.status.as_u16(), 403 | 406 | 419 | 429 | 503)
            && let Ok(second) = get(&self.browser, url, Limit::Page).await
            && second.status.is_success()
        {
            return Ok(second);
        }
        Ok(first)
    }

    /// Fetches a preview image. A page names its images itself, so one could point Knoa at
    /// this machine or the local network; those aren't fetched. Redirects there are
    /// stopped by the client.
    pub async fn image(&self, url: &Url) -> Result<Response, Error> {
        if access::is_private(url) {
            return Err(Error::PrivateAddress);
        }
        get(&self.knoa, url, Limit::Image).await
    }

    /// `None` for a non-success status, which for site APIs means "not something we know".
    pub async fn get_json<T: DeserializeOwned>(&self, url: &Url) -> Result<Option<T>, Error> {
        let response = get(&self.knoa, url, Limit::Page).await?;
        if !response.status.is_success() {
            return Ok(None);
        }
        Ok(serde_json::from_slice(&response.body).ok())
    }
}

/// The buffer a body starts with: its announced length, up to the limit. A body that
/// grows by doubling can sit in nearly twice its size while it is decoded.
fn capacity(announced: Option<u64>, limit: usize) -> usize {
    announced.map_or(0, |length| {
        usize::try_from(length).unwrap_or(limit).min(limit)
    })
}

async fn get(client: &Client, url: &Url, limit: Limit) -> Result<Response, Error> {
    let response = client.get(url.clone()).timeout(TIMEOUT).send().await?;
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let final_url = response.url().clone();
    let status = response.status();
    let whole_or_nothing = limit.whole_or_nothing(&content_type);
    let limit = limit.bytes(&content_type);
    let (body, truncated) = match response.content_length() {
        // Not worth downloading only to cut off.
        Some(length) if whole_or_nothing && length > limit as u64 => (Vec::new(), true),
        _ => platform::read_body(response, limit).await?,
    };
    Ok(Response {
        url: final_url,
        status,
        content_type,
        body,
        truncated,
    })
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use tokio::{
        io::{AsyncReadExt as _, AsyncWriteExt as _},
        net::TcpListener,
    };

    use super::*;

    /// Serves one response with `body`, announcing its length only when asked.
    async fn serve(content_type: &'static str, body: Vec<u8>, with_length: bool) -> Url {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("a free port");
        let address = listener.local_addr().expect("bound");
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("a connection");
            let mut request = [0; 1024];
            let _ = socket.read(&mut request).await;
            let length = if with_length {
                format!("Content-Length: {}\r\n", body.len())
            } else {
                String::new()
            };
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\n{length}Connection: close\r\n\r\n"
            );
            // The client stops reading at its limit and hangs up, which fails these writes.
            let _ = socket.write_all(head.as_bytes()).await;
            let _ = socket.write_all(&body).await;
        });
        Url::parse(&format!("http://{address}/")).expect("valid URL")
    }

    #[tokio::test]
    async fn a_page_over_the_limit_keeps_its_start_whether_or_not_its_length_is_known() {
        let mut page = b"<html><head><title>Big page</title></head><body>".to_vec();
        page.resize(PAGE_LIMIT + 1024, b'x');
        let http = Http::new().expect("a client");

        for with_length in [true, false] {
            let url = serve("text/html", page.clone(), with_length).await;
            let response = http.page(&url).await.expect("a response");

            assert!(response.truncated, "with Content-Length: {with_length}");
            assert_eq!(
                response.body.len(),
                PAGE_LIMIT,
                "with Content-Length: {with_length}"
            );
            assert!(response.body.starts_with(b"<html><head><title>Big page"));
        }
    }

    #[tokio::test]
    async fn an_image_announced_over_the_limit_is_not_downloaded() {
        let image = vec![0; RAW_IMAGE_LIMIT + 1];
        let http = Http::new().expect("a client");

        let url = serve("image/jpeg", image.clone(), true).await;
        let announced = http.page(&url).await.expect("a response");
        let url = serve("image/jpeg", image, false).await;
        let unannounced = http.page(&url).await.expect("a response");

        assert!(announced.truncated && announced.body.is_empty());
        assert!(unannounced.truncated && unannounced.body.len() == RAW_IMAGE_LIMIT);
    }
}
