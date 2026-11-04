//! Downloading preview images into the thumbnail cache.

use std::{
    hash::{DefaultHasher, Hash as _, Hasher as _},
    path::Path,
};

use knoa_core::Image;
use url::Url;

use crate::http::{Http, IMAGE_LIMIT};

/// Smaller images are icons or tracking pixels, not previews.
const MIN_WIDTH: usize = 120;

/// Downloads `url` into `cache_dir`, keyed by URL, and reads its size. `None` when it isn't
/// a usable preview image.
pub async fn download(http: &Http, url: &Url, cache_dir: &Path) -> Option<Image> {
    let mut hasher = DefaultHasher::new();
    url.as_str().hash(&mut hasher);
    let stem = format!("{:016x}", hasher.finish());

    let response = http
        .bytes(url, IMAGE_LIMIT)
        .await
        .inspect_err(|err| tracing::debug!(%url, "image fetch failed: {err}"))
        .ok()?;
    if !response.status.is_success() {
        return None;
    }
    let size = imagesize::blob_size(&response.body).ok()?;
    if size.width < MIN_WIDTH {
        return None;
    }
    let extension = match imagesize::image_type(&response.body).ok()? {
        imagesize::ImageType::Png => "png",
        imagesize::ImageType::Jpeg => "jpg",
        imagesize::ImageType::Gif => "gif",
        imagesize::ImageType::Webp => "webp",
        _ => return None,
    };
    let path = cache_dir.join(format!("{stem}.{extension}"));
    tokio::fs::create_dir_all(cache_dir).await.ok()?;
    tokio::fs::write(&path, &response.body)
        .await
        .inspect_err(|err| tracing::warn!(?path, "can't cache image: {err}"))
        .ok()?;
    Some(Image {
        path,
        width: size.width as u32,
        height: size.height as u32,
    })
}
