//! Images: downloading a page's preview image, and reading a link straight to an image
//! file into a display copy and a copy for the AI.

mod decode;
mod encode;
mod facts;
mod raw;
mod scale;

use std::fmt;

pub use facts::Facts;
pub use raw::{RawImage, read as read_raw};
use url::Url;

use crate::http::Http;

/// Smaller images are icons or tracking pixels, not previews.
const MIN_WIDTH: usize = 120;

/// Shorter images are strips or banners. Still low enough for a 200x112 video still.
const MIN_HEIGHT: usize = 90;

/// Wider or taller than this (long side over short side) is a logo or banner, not a
/// preview. Cards crop to about 16:9, so these would be unrecognizable slivers anyway.
const MAX_ASPECT: usize = 4;

/// An image for the bookmark's card and detail, not yet stored.
#[derive(Clone)]
pub struct Preview {
    /// Where it was downloaded from. Copies of the same image share it.
    pub url: Url,
    pub width: u32,
    pub height: u32,
    /// `png`, `jpg`, `gif` or `webp`.
    pub extension: &'static str,
    pub bytes: Vec<u8>,
}

/// Without the bytes, which would bury a log line.
impl fmt::Debug for Preview {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Preview")
            .field("url", &self.url.as_str())
            .field("width", &self.width)
            .field("height", &self.height)
            .field("extension", &self.extension)
            .field("bytes", &self.bytes.len())
            .finish()
    }
}

/// Downloads a page's preview image. `None` when it isn't shaped like a preview.
pub async fn download_preview(http: &Http, url: &Url) -> Option<Preview> {
    download(http, url)
        .await
        .filter(|p| is_preview_shaped(p.width as usize, p.height as usize))
}

/// Downloads `url` and reads its size, whatever its shape: an image already chosen as a
/// bookmark's, like a panorama linked straight to. `None` when it isn't a usable image.
pub async fn download(http: &Http, url: &Url) -> Option<Preview> {
    let response = http
        .image(url)
        .await
        .inspect_err(|err| tracing::debug!(%url, "image fetch failed: {err}"))
        .ok()?;
    // A cut-off file still has a valid header, and would be stored and shown broken.
    if !response.status.is_success() || response.truncated {
        return None;
    }
    let size = imagesize::blob_size(&response.body).ok()?;
    let extension = match imagesize::image_type(&response.body).ok()? {
        imagesize::ImageType::Png => "png",
        imagesize::ImageType::Jpeg => "jpg",
        imagesize::ImageType::Gif => "gif",
        imagesize::ImageType::Webp => "webp",
        _ => return None,
    };
    Some(Preview {
        url: url.clone(),
        width: size.width as u32,
        height: size.height as u32,
        extension,
        bytes: response.body,
    })
}

fn is_preview_shaped(width: usize, height: usize) -> bool {
    width >= MIN_WIDTH
        && height >= MIN_HEIGHT
        && width <= height * MAX_ASPECT
        && height <= width * MAX_ASPECT
}
