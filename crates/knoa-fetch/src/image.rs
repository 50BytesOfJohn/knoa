//! Downloading preview images and checking they are shaped like previews.

use url::Url;

use crate::http::{Http, IMAGE_LIMIT};

/// Smaller images are icons or tracking pixels, not previews.
const MIN_WIDTH: usize = 120;

/// Shorter images are strips or banners. Still low enough for a 200x112 video still.
const MIN_HEIGHT: usize = 90;

/// Wider or taller than this (long side over short side) is a logo or banner, not a
/// preview. Cards crop to about 16:9, so these would be unrecognizable slivers anyway.
const MAX_ASPECT: usize = 4;

/// A downloaded preview image, not yet stored.
pub struct Preview {
    /// Where it was downloaded from.
    pub url: Url,
    pub width: u32,
    pub height: u32,
    /// `png`, `jpg`, `gif` or `webp`.
    pub extension: &'static str,
    pub bytes: Vec<u8>,
}

/// Downloads `url` and reads its size. `None` when it isn't a usable preview image.
pub async fn download(http: &Http, url: &Url) -> Option<Preview> {
    let response = http
        .bytes(url, IMAGE_LIMIT)
        .await
        .inspect_err(|err| tracing::debug!(%url, "image fetch failed: {err}"))
        .ok()?;
    if !response.status.is_success() {
        return None;
    }
    let size = imagesize::blob_size(&response.body).ok()?;
    if !is_preview_shaped(size.width, size.height) {
        return None;
    }
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
