//! The desktop's fetch: preview images go into the thumbnail cache on disk.

use std::{
    hash::{DefaultHasher, Hash as _, Hasher as _},
    path::{Path, PathBuf},
};

use knoa_core::{Image, Page};

use crate::{Error, Fetched, Fetcher, Preview, http};

impl Fetcher {
    pub fn new(image_cache: PathBuf) -> Result<Fetcher, Error> {
        Ok(Fetcher {
            http: http::Http::new()?,
            image_cache,
        })
    }

    /// [`Fetcher::fetch_with_preview`], with the preview stored in the thumbnail cache.
    pub async fn fetch(&self, url: &str) -> Result<Page, Error> {
        let Fetched { mut page, preview } = self.fetch_with_preview(url).await?;
        if let Some(preview) = preview {
            page.image = store(&preview, &self.image_cache).await;
        }
        Ok(page)
    }
}

/// Writes `preview` into `cache_dir`, keyed by its URL.
async fn store(preview: &Preview, cache_dir: &Path) -> Option<Image> {
    let mut hasher = DefaultHasher::new();
    preview.url.as_str().hash(&mut hasher);
    let stem = format!("{:016x}", hasher.finish());
    let path = cache_dir.join(format!("{stem}.{}", preview.extension));
    tokio::fs::create_dir_all(cache_dir).await.ok()?;
    tokio::fs::write(&path, &preview.bytes)
        .await
        .inspect_err(|err| tracing::warn!(?path, "can't cache image: {err}"))
        .ok()?;
    Some(Image {
        path,
        width: preview.width,
        height: preview.height,
    })
}
