//! Dragging onto the sidebar's folders: a folder to nest it, reorder it or move it to the
//! top level, and a bookmark to file it.

use std::{rc::Rc, time::Duration};

use gpui::{Bounds, Context, DragMoveEvent, Pixels, Point, ScrollHandle, SharedString, Task};
use knoa_core::{BookmarkId, FolderId, FolderTree};

use super::{
    SidebarView,
    outline::{Disclosure, FolderRow},
};

/// How long a drag rests on a collapsed parent before it opens.
const HOVER_TO_EXPAND: Duration = Duration::from_millis(600);
/// The share of a row's height at its top and bottom that drops between rows.
const EDGE: f32 = 0.28;

/// A folder being dragged.
#[derive(Debug, Clone)]
pub(in crate::ui::main) struct DraggedFolder(pub Rc<FolderRow>);

/// A bookmark being dragged from the cards.
#[derive(Debug, Clone)]
pub(in crate::ui::main) struct DraggedBookmark {
    pub id: BookmarkId,
    pub title: SharedString,
}

/// Where a drag would land.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DropSpot {
    Into(FolderId),
    Before(FolderId),
    After(FolderId),
    /// Last at the top level.
    End,
}

/// The sidebar's part of a drag in progress.
#[derive(Default)]
pub(in crate::ui::main) struct SidebarDrop {
    pub(super) scroll: ScrollHandle,
    spot: Option<DropSpot>,
    expanding: Option<(FolderId, Task<()>)>,
}

impl SidebarDrop {
    // DragMoveEvent carries the unmasked row bounds, including the part scrolled away.
    fn contains(&self, bounds: Bounds<Pixels>, position: Point<Pixels>) -> bool {
        self.scroll.bounds().contains(&position) && bounds.contains(&position)
    }

    pub(super) fn spot(&self) -> Option<DropSpot> {
        self.spot
    }

    /// Forgets a drag that ended anywhere, so its marks don't linger.
    pub(super) fn end(&mut self) {
        self.spot = None;
        self.expanding = None;
    }
}

/// Where `folder` would go for `spot`: its new parent and the sibling it goes before.
/// `None` where it can't go, so no drop mark shows there.
pub(super) fn placement(
    tree: &FolderTree,
    folder: FolderId,
    spot: DropSpot,
) -> Option<(Option<FolderId>, Option<FolderId>)> {
    let (parent, before) = match spot {
        DropSpot::Into(target) => (Some(target), None),
        DropSpot::Before(target) => (tree.parent(target).map(|f| f.id), Some(target)),
        DropSpot::After(target) => {
            let parent = tree.parent(target).map(|f| f.id);
            let next = tree
                .children(parent)
                .skip_while(|f| f.id != target)
                .nth(1)
                .map(|f| f.id);
            (parent, next)
        }
        DropSpot::End => (None, None),
    };
    let onto_itself = match spot {
        DropSpot::Into(target) | DropSpot::Before(target) | DropSpot::After(target) => {
            target == folder
        }
        DropSpot::End => false,
    };
    (!onto_itself && tree.check_move(folder, parent).is_ok()).then_some((parent, before))
}

/// The spot under the pointer in `row`: its edges drop between rows, its middle into it.
/// An open parent's lower edge drops into it, where its first child shows. When the
/// folder can't go into the row, the whole row drops before or after it.
fn spot_in_row(
    tree: &FolderTree,
    folder: FolderId,
    row: &FolderRow,
    position: Point<Pixels>,
    top: Pixels,
    height: Pixels,
) -> Option<DropSpot> {
    let at = ((position.y - top) / height).clamp(0., 1.);
    let edge = if at < EDGE {
        Some(DropSpot::Before(row.id))
    } else if at > 1. - EDGE && row.disclosure != Disclosure::Expanded {
        Some(DropSpot::After(row.id))
    } else {
        None
    };
    let half = if at < 0.5 {
        DropSpot::Before(row.id)
    } else {
        DropSpot::After(row.id)
    };
    let fits = |spot: DropSpot| placement(tree, folder, spot).is_some();
    [edge, Some(DropSpot::Into(row.id)), Some(half)]
        .into_iter()
        .flatten()
        .find(|&spot| fits(spot))
}

impl SidebarView {
    pub(super) fn drag_folder_over_row(
        &mut self,
        row: &FolderRow,
        event: &DragMoveEvent<DraggedFolder>,
        cx: &mut Context<'_, SidebarView>,
    ) {
        let bounds = event.bounds;
        let spot = if self.drop.contains(bounds, event.event.position) {
            let dragged = event.drag(cx).0.id;
            let tree = self.library.read(cx).folder_tree();
            spot_in_row(
                tree,
                dragged,
                row,
                event.event.position,
                bounds.top(),
                bounds.size.height,
            )
        } else {
            self.spot_leaving(row.id)
        };
        self.set_drop_spot(spot, row, cx);
    }

    pub(super) fn drag_bookmark_over_row(
        &mut self,
        row: &FolderRow,
        event: &DragMoveEvent<DraggedBookmark>,
        cx: &mut Context<'_, SidebarView>,
    ) {
        let spot = if self.drop.contains(event.bounds, event.event.position) {
            Some(DropSpot::Into(row.id))
        } else {
            self.spot_leaving(row.id)
        };
        self.set_drop_spot(spot, row, cx);
    }

    /// The spot after the pointer left a row: cleared if it was that row's, kept if
    /// another row has already taken it.
    fn spot_leaving(&self, row: FolderId) -> Option<DropSpot> {
        match self.drop.spot {
            Some(DropSpot::Into(id) | DropSpot::Before(id) | DropSpot::After(id)) if id == row => {
                None
            }
            spot => spot,
        }
    }

    pub(super) fn drag_folder_over_end(
        &mut self,
        event: &DragMoveEvent<DraggedFolder>,
        cx: &mut Context<'_, SidebarView>,
    ) {
        let spot = if self.drop.contains(event.bounds, event.event.position) {
            Some(DropSpot::End)
        } else if self.drop.spot == Some(DropSpot::End) {
            None
        } else {
            return;
        };
        if self.drop.spot != spot {
            self.drop.expanding = None;
            self.drop.spot = spot;
            cx.notify();
        }
    }

    fn set_drop_spot(
        &mut self,
        spot: Option<DropSpot>,
        row: &FolderRow,
        cx: &mut Context<'_, SidebarView>,
    ) {
        if self.drop.spot == spot {
            return;
        }
        self.drop.expanding = None;
        self.drop.spot = spot;
        if spot == Some(DropSpot::Into(row.id)) && row.disclosure == Disclosure::Collapsed {
            self.expand_after_hover(row.id, cx);
        }
        cx.notify();
    }

    /// Opens a collapsed parent the drag has rested on, like Finder's spring-loading.
    fn expand_after_hover(&mut self, id: FolderId, cx: &mut Context<'_, SidebarView>) {
        if self
            .drop
            .expanding
            .as_ref()
            .is_some_and(|(at, _)| *at == id)
        {
            return;
        }
        let task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(HOVER_TO_EXPAND).await;
            let _ = this.update(cx, |this, cx| {
                this.drop.expanding = None;
                if cx.has_active_drag() && this.drop.spot == Some(DropSpot::Into(id)) {
                    this.expand(id, cx);
                }
            });
        });
        self.drop.expanding = Some((id, task));
    }

    pub(super) fn drop_folder(&mut self, folder: FolderId, cx: &mut Context<'_, SidebarView>) {
        let Some(spot) = self.drop.spot.take() else {
            return;
        };
        let tree = self.library.read(cx).folder_tree();
        let Some((parent, before)) = placement(tree, folder, spot) else {
            return;
        };
        if let Some(parent) = parent {
            self.expand(parent, cx);
        }
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| store.place_folder(folder, parent, before));
        });
    }

    pub(super) fn drop_bookmark(
        &mut self,
        bookmark: BookmarkId,
        cx: &mut Context<'_, SidebarView>,
    ) {
        if let Some(DropSpot::Into(folder)) = self.drop.spot.take() {
            self.library.update(cx, |library, cx| {
                library.change(cx, move |store| store.add_to_folder(bookmark, folder));
            });
        }
    }
}
