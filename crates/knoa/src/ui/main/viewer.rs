//! The full-window viewer for image bookmarks, like Quick Look: the stored picture on a
//! dark scrim with its caption and actions, moved through with the arrow keys.

mod view;

use std::time::Duration;

use gpui::{Context, Task, Window};
use knoa_core::BookmarkId;

use super::{
    CloseViewer, CopyImage, MainView, NextImage, Overlay, PreviousImage, QuickLook, SaveImage,
    picture::has_picture,
};

/// How long a word about the last action stays under the caption.
const NOTICE_FOR: Duration = Duration::from_millis(1600);

pub struct Viewer {
    pub bookmark: BookmarkId,
    /// What the last action did, until its task clears it.
    notice: Option<(&'static str, Task<()>)>,
}

impl Viewer {
    pub fn notice(&self) -> Option<&'static str> {
        self.notice.as_ref().map(|(text, _)| *text)
    }
}

impl MainView {
    /// Shows the bookmark's picture, if it has one Knoa kept. The viewer takes the
    /// selection along, so closing it leaves the last picture selected in the grid.
    pub(super) fn open_viewer(
        &mut self,
        id: BookmarkId,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        if self.picture_of(id, cx).is_none() {
            return;
        }
        self.set_selected(Some(id), cx);
        let viewer = Viewer {
            bookmark: id,
            notice: None,
        };
        self.set_overlay(Some(Overlay::Viewer(viewer)), cx);
        self.viewer_focus.focus(window, cx);
    }

    /// Space: the viewer for a picture, the detail panel for anything else.
    pub(super) fn quick_look(
        &mut self,
        _: &QuickLook,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        match self.selected {
            Some(id) if self.picture_of(id, cx).is_some() => self.open_viewer(id, window, cx),
            _ => self.toggle_detail(cx),
        }
    }

    /// Closes the viewer before an app-wide action moves focus elsewhere: behind it the
    /// grid would take keys while the viewer still showed another picture.
    pub(super) fn close_viewer_first(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        if self.viewed().is_some() {
            self.close_overlay(window, cx);
        }
    }

    /// The bookmark the viewer shows, while it is open.
    pub(super) fn viewed(&self) -> Option<BookmarkId> {
        match &self.overlay {
            Some(Overlay::Viewer(viewer)) => Some(viewer.bookmark),
            _ => None,
        }
    }

    /// Says briefly what an action did, when the viewer is open to say it. Elsewhere the
    /// clipboard or the saved file speaks for itself.
    pub(super) fn tell_viewer(&mut self, text: &'static str, cx: &mut Context<'_, MainView>) {
        let Some(Overlay::Viewer(viewer)) = &mut self.overlay else {
            return;
        };
        let clear = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(NOTICE_FOR).await;
            let _ = this.update(cx, |this, cx| {
                if let Some(Overlay::Viewer(viewer)) = &mut this.overlay {
                    viewer.notice = None;
                }
                cx.notify();
            });
        });
        viewer.notice = Some((text, clear));
        cx.notify();
    }

    /// The pictures the arrows move through: those among the results, in grid order.
    fn viewable(&self) -> Vec<BookmarkId> {
        let results = &self.layout.results;
        self.layout
            .layout
            .cards
            .iter()
            .filter_map(|card| results.get(card.index))
            .filter(|bookmark| has_picture(bookmark))
            .map(|bookmark| bookmark.id)
            .collect()
    }

    fn step_viewer(&mut self, forward: bool, window: &mut Window, cx: &mut Context<'_, MainView>) {
        let Some(current) = self.viewed() else {
            return;
        };
        let pictures = self.viewable();
        let Some(at) = pictures.iter().position(|id| *id == current) else {
            return;
        };
        let next = if forward {
            at.checked_add(1)
        } else {
            at.checked_sub(1)
        };
        let Some(&id) = next.and_then(|next| pictures.get(next)) else {
            return;
        };
        if let Some(Overlay::Viewer(viewer)) = &mut self.overlay {
            viewer.bookmark = id;
            viewer.notice = None;
        }
        self.set_selected(Some(id), cx);
        self.reveal_bookmark(id, window);
        cx.notify();
    }

    fn close_viewer(&mut self, _: &CloseViewer, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.close_overlay(window, cx);
    }

    fn previous_image(
        &mut self,
        _: &PreviousImage,
        window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        self.step_viewer(false, window, cx);
    }

    fn next_image(&mut self, _: &NextImage, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.step_viewer(true, window, cx);
    }

    fn copy_image(&mut self, _: &CopyImage, _: &mut Window, cx: &mut Context<'_, Self>) {
        if let Some(id) = self.viewed() {
            self.copy_image_of(id, cx);
        }
    }

    fn save_image(&mut self, _: &SaveImage, _: &mut Window, cx: &mut Context<'_, Self>) {
        if let Some(id) = self.viewed() {
            self.save_image_of(id, cx);
        }
    }
}
