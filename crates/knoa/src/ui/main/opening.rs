//! Opening bookmarks in the browser, in front of Knoa or behind it.

use std::time::{Duration, Instant};

use gpui::{Context, Window};
use knoa_browser::Activation;
use knoa_core::BookmarkId;

use super::{MainView, OpenSelected, OpenSelectedInBackground};
use crate::services::links;

/// How long after a background open Knoa takes focus back. Chromium browsers bring
/// themselves to the front a few hundred milliseconds after `open -g` returns.
const KEEP_FOCUS_FOR: Duration = Duration::from_millis(1500);

impl MainView {
    pub(super) fn open_selected(
        &mut self,
        _: &OpenSelected,
        _: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        if let Some(id) = self.selected {
            self.open_bookmark(id, Activation::Foreground, cx);
        }
    }

    pub(super) fn open_selected_in_background(
        &mut self,
        _: &OpenSelectedInBackground,
        _: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        if let Some(id) = self.selected {
            self.open_bookmark(id, Activation::Background, cx);
        }
    }

    pub(super) fn open_bookmark(
        &mut self,
        id: BookmarkId,
        activation: Activation,
        cx: &mut Context<'_, MainView>,
    ) {
        if let Some(url) = self.library.read(cx).bookmark(id).map(|b| b.url.clone()) {
            self.open_link(&url, activation, cx);
        }
    }

    /// Opens a link in the browser the current workspace picked.
    pub(super) fn open_link(
        &mut self,
        url: &str,
        activation: Activation,
        cx: &mut Context<'_, MainView>,
    ) {
        let opens_in = self
            .library
            .read(cx)
            .workspace()
            .and_then(|w| w.opens_in.clone());
        self.keep_focus_until =
            (activation == Activation::Background).then(|| Instant::now() + KEEP_FOCUS_FOR);
        links::open(url, opens_in.as_deref(), activation, cx);
    }

    /// Takes focus back when the browser grabs it right after a background open.
    pub(super) fn keep_focus(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        if window.is_window_active() {
            return;
        }
        if self
            .keep_focus_until
            .take()
            .is_some_and(|until| Instant::now() < until)
        {
            cx.activate(true);
        }
    }
}
