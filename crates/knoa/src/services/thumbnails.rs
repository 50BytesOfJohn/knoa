//! Fetching a bookmark's thumbnail again when the user asks, outside the organizing queue
//! so it neither waits behind a long import nor touches what the AI wrote.

use std::{collections::HashMap, sync::Arc, time::Duration};

use gpui::{App, AppContext as _, Context, Entity, Global, ImageSource, Task};
use knoa_core::{Bookmark, BookmarkId, Image};
use knoa_fetch::Fetcher;
use knoa_store::BookmarkIdentity;

use crate::{services::network::Network, state::library::Library};

/// How long a refresh that found nothing says so.
const SHOW_OUTCOME_FOR: Duration = Duration::from_secs(4);

/// Where a bookmark's refresh stands. Only the last refresh is remembered, and only briefly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refresh {
    Running,
    /// The page has no image that works as a preview.
    NoImage,
    /// The page couldn't be fetched.
    Failed,
}

pub struct Thumbnails {
    library: Entity<Library>,
    fetcher: Arc<Fetcher>,
    /// Each refresh with the task that runs it, or that later forgets its outcome.
    refreshes: HashMap<BookmarkId, (Refresh, Task<()>)>,
}

struct ThumbnailsGlobal(Entity<Thumbnails>);

impl Global for ThumbnailsGlobal {}

impl Thumbnails {
    pub fn init(library: Entity<Library>, fetcher: Arc<Fetcher>, cx: &mut App) {
        let thumbnails = cx.new(|_| Thumbnails {
            library,
            fetcher,
            refreshes: HashMap::new(),
        });
        cx.set_global(ThumbnailsGlobal(thumbnails));
    }

    pub fn entity(cx: &App) -> Entity<Thumbnails> {
        cx.global::<ThumbnailsGlobal>().0.clone()
    }

    pub fn refresh_of(&self, id: BookmarkId) -> Option<Refresh> {
        self.refreshes.get(&id).map(|(refresh, _)| *refresh)
    }

    /// Fetches the bookmark's page again and keeps its image if it has one. A page without
    /// one leaves the old thumbnail in place.
    pub fn refresh(&mut self, bookmark: Bookmark, cx: &mut Context<'_, Thumbnails>) {
        let id = bookmark.id;
        if self.refresh_of(id) == Some(Refresh::Running) {
            return;
        }
        let fetcher = self.fetcher.clone();
        let database = self.library.read(cx).database();
        let url = bookmark.url.clone();
        let identity =
            cx.background_spawn(async move { database.read().bookmark_identity(&bookmark) });
        let network = Network::handle(cx);
        let task = cx.spawn(async move |this, cx| {
            let identity = match identity.await {
                Ok(Some(identity)) => identity,
                result => {
                    if let Err(err) = result {
                        tracing::error!(?id, "can't read thumbnail identity: {err}");
                    }
                    let _ = this.update(cx, |this, cx| this.finish(id, Err(Refresh::Failed), cx));
                    return;
                }
            };
            let outcome = match network
                .spawn(async move { fetcher.fetch(&url).await })
                .await
            {
                Ok(Ok(page)) => page
                    .image
                    .map(|image| (identity, image))
                    .ok_or(Refresh::NoImage),
                Ok(Err(err)) => {
                    tracing::info!(?id, "can't refresh the thumbnail: {err}");
                    Err(Refresh::Failed)
                }
                Err(err) => {
                    tracing::error!(?id, "thumbnail task failed: {err}");
                    Err(Refresh::Failed)
                }
            };
            let _ = this.update(cx, |this, cx| this.finish(id, outcome, cx));
        });
        self.refreshes.insert(id, (Refresh::Running, task));
        cx.notify();
    }

    fn finish(
        &mut self,
        id: BookmarkId,
        outcome: Result<(BookmarkIdentity, Image), Refresh>,
        cx: &mut Context<'_, Thumbnails>,
    ) {
        match outcome {
            Ok((identity, image)) => {
                self.refreshes.remove(&id);
                // The cache is keyed by image URL, so a new picture can land in the same
                // file; GPUI would keep drawing the one it decoded before.
                ImageSource::from(image.path.clone()).remove_asset(cx);
                self.library.update(cx, |library, cx| {
                    library.change(cx, move |store| {
                        store.set_image_if_current(&identity, &image)
                    });
                });
            }
            Err(refresh) => {
                let forget = cx.spawn(async move |this, cx| {
                    cx.background_executor().timer(SHOW_OUTCOME_FOR).await;
                    let _ = this.update(cx, |this, cx| {
                        this.refreshes.remove(&id);
                        cx.notify();
                    });
                });
                self.refreshes.insert(id, (refresh, forget));
            }
        }
        cx.notify();
    }
}
