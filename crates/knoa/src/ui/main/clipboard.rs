//! Copying a bookmark's link, confirmed for a moment on the detail panel.

use std::time::Duration;

use gpui::{ClipboardItem, Context, Window};
use knoa_core::BookmarkId;

use super::{CopyLink, MainView};

impl MainView {
    pub(super) fn copy_link(
        &mut self,
        _: &CopyLink,
        _: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        if let Some(id) = self.selected {
            self.copy_link_of(id, cx);
        }
    }

    pub(super) fn copy_link_of(&mut self, id: BookmarkId, cx: &mut Context<'_, MainView>) {
        if let Some(url) = self.url_of(id, cx) {
            self.copy_to_clipboard(id, url, cx);
        }
    }

    /// Copies a bookmark's link and confirms it on the detail panel's copy button for a
    /// moment.
    pub(super) fn copy_to_clipboard(
        &mut self,
        id: BookmarkId,
        url: String,
        cx: &mut Context<'_, MainView>,
    ) {
        cx.write_to_clipboard(ClipboardItem::new_string(url));
        let reset = cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(1500))
                .await;
            let _ = this.update(cx, |this, cx| {
                this.copied = None;
                cx.notify();
            });
        });
        self.copied = Some((id, reset));
        cx.notify();
    }
}
