//! Thumbnails between this Mac's cache and the server: ones another device stored are
//! downloaded, and ones this Mac's organizer fetched are uploaded when it organizes for
//! the account. A few at a time, and a thumbnail that fails is left alone until the next
//! launch rather than tried every round.

use std::collections::HashSet;

use futures::{StreamExt as _, stream};
use gpui::AsyncApp;
use knoa_core::BookmarkId;

use super::engine::{Engine, Failure, net};
use crate::state::library::Library;

const BATCH: usize = 24;
const AT_ONCE: usize = 4;

#[derive(Default)]
pub(super) struct Thumbnails {
    failed: HashSet<BookmarkId>,
}

impl Thumbnails {
    pub async fn download(&mut self, engine: &Engine, cx: &mut AsyncApp) -> Result<(), Failure> {
        loop {
            let limit = BATCH + self.failed.len();
            let mut wanted = engine
                .store(cx, move |store| store.thumbnail_downloads(limit))
                .await?;
            wanted.retain(|d| !self.failed.contains(&d.id));
            if wanted.is_empty() {
                return Ok(());
            }
            let fetches = wanted.into_iter().map(|download| {
                let fetcher = engine.fetcher.clone();
                let url = engine.api.image_url(&download.image_key);
                let fetch = net(cx, async move { Ok(fetcher.download_image(&url?).await) });
                async move { (download.id, fetch.await) }
            });
            let fetched: Vec<_> = stream::iter(fetches)
                .buffer_unordered(AT_ONCE)
                .collect()
                .await;
            for (id, image) in fetched {
                match image {
                    Ok(Some(image)) => {
                        engine
                            .store(cx, move |store| store.set_image(id, &image))
                            .await?;
                    }
                    Ok(None) => {
                        tracing::debug!(?id, "server thumbnail isn't a usable image");
                        self.failed.insert(id);
                    }
                    Err(failure) => return Err(failure),
                }
            }
            engine.library.update(cx, Library::refresh_soon);
        }
    }

    pub async fn upload(&mut self, engine: &Engine, cx: &mut AsyncApp) -> Result<(), Failure> {
        loop {
            let limit = BATCH + self.failed.len();
            let mut wanted = engine
                .store(cx, move |store| store.thumbnail_uploads(limit))
                .await?;
            wanted.retain(|u| !self.failed.contains(&u.id));
            if wanted.is_empty() {
                return Ok(());
            }
            let sends = wanted.into_iter().map(|upload| {
                let api = engine.api.clone();
                let send = net(cx, async move {
                    let bytes = tokio::fs::read(&upload.path).await?;
                    api.upload_image(&upload.bookmark, bytes, upload.width, upload.height)
                        .await
                });
                async move { (upload.id, send.await) }
            });
            let sent: Vec<_> = stream::iter(sends)
                .buffer_unordered(AT_ONCE)
                .collect()
                .await;
            for (id, answer) in sent {
                match answer {
                    Ok(answer) => {
                        engine
                            .store(cx, move |store| store.set_image_key(id, &answer.image_key))
                            .await?;
                    }
                    Err(Failure::Sync(err)) if !err.is_transient() || is_missing_file(&err) => {
                        tracing::info!(?id, "can't upload the thumbnail: {err}");
                        self.failed.insert(id);
                    }
                    Err(failure) => return Err(failure),
                }
            }
        }
    }
}

/// The launch sweep may have deleted a cached file a row still names.
fn is_missing_file(err: &knoa_sync::Error) -> bool {
    matches!(err, knoa_sync::Error::Io(io) if io.kind() == std::io::ErrorKind::NotFound)
}
