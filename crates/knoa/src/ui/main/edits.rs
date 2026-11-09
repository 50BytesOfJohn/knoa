//! Changes to one bookmark that several parts of the main screen ask for: Up next,
//! folders, organizing and its thumbnail.

use gpui::Context;
use knoa_core::{BookmarkId, FolderId};

use super::MainView;
use crate::services::{organizer::Organizer, thumbnails::Thumbnails};

impl MainView {
    pub fn set_up_next(&mut self, id: BookmarkId, cx: &mut Context<'_, MainView>) {
        let Some(on) = self.library.read(cx).bookmark(id).map(|b| !b.up_next) else {
            return;
        };
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| store.set_up_next(id, on));
        });
    }

    pub fn set_in_folder(
        &mut self,
        id: BookmarkId,
        folder: FolderId,
        in_folder: bool,
        cx: &mut Context<'_, MainView>,
    ) {
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| {
                if in_folder {
                    store.add_to_folder(id, folder)
                } else {
                    store.remove_from_folder(id, folder)
                }
            });
        });
    }

    /// Queues the bookmark for the AI, also when it was organized before.
    pub fn organize(&mut self, id: BookmarkId, cx: &mut Context<'_, MainView>) {
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| store.queue(id));
        });
        Organizer::wake(cx);
    }

    pub fn refresh_thumbnail(&mut self, id: BookmarkId, cx: &mut Context<'_, MainView>) {
        if let Some(url) = self.url_of(id, cx) {
            Thumbnails::entity(cx).update(cx, |thumbnails, cx| thumbnails.refresh(id, url, cx));
        }
    }
}
