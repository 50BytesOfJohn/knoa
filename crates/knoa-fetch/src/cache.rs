//! The desktop's fetch: preview images go into the thumbnail cache on disk.

use std::{
    hash::{DefaultHasher, Hash as _, Hasher as _},
    path::{Path, PathBuf},
};

use knoa_core::{AnalysisImage, Image, Page};

use crate::{Error, Fetched, Fetcher, Preview, http};

/// A fetched page whose preview is in the thumbnail cache, as `page.image`.
pub struct CachedPage {
    pub page: Page,
    /// For a link straight to an image file: a smaller copy for the AI to look at.
    pub analysis: Option<AnalysisImage>,
}

impl Fetcher {
    pub fn new(image_cache: PathBuf) -> Result<Fetcher, Error> {
        Ok(Fetcher {
            http: http::Http::new()?,
            image_cache,
        })
    }

    /// [`Fetcher::fetch_with_preview`], with the preview stored in the thumbnail cache.
    pub async fn fetch(&self, url: &str) -> Result<CachedPage, Error> {
        Ok(self.cache(self.fetch_with_preview(url).await?).await)
    }

    /// [`Fetcher::fetch_snapshot`], with the preview stored in the thumbnail cache.
    pub async fn fetch_snapshot_cached(
        &self,
        url: &str,
        html: String,
    ) -> Result<CachedPage, Error> {
        Ok(self.cache(self.fetch_snapshot(url, html).await?).await)
    }

    async fn cache(&self, fetched: Fetched) -> CachedPage {
        let Fetched {
            mut page,
            preview,
            analysis,
        } = fetched;
        if let Some(preview) = preview {
            page.image = store(&preview, &self.image_cache).await;
        }
        CachedPage { page, analysis }
    }

    /// Downloads a bookmark's image on its own, like a thumbnail another device stored on
    /// the server, into the thumbnail cache. `None` when it isn't a usable image.
    pub async fn download_image(&self, url: &url::Url) -> Option<Image> {
        let preview = crate::image::download(&self.http, url).await?;
        store(&preview, &self.image_cache).await
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
