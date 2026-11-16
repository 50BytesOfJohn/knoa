//! Dragging onto the sidebar's folders: a folder to nest it, reorder it or move it to the
//! top level, and a bookmark to file it.

use std::{rc::Rc, time::Duration};

use gpui::{
    AppContext as _, Context, DragMoveEvent, FontWeight, IntoElement, ParentElement as _, Pixels,
    Point, Render, SharedString, Styled as _, Task, Window, div, px,
};
use knoa_core::{BookmarkId, FolderIcon, FolderId, FolderTree};

use super::outline::{Disclosure, FolderRow};
use crate::ui::{
    components::{Finish, folder_mark, icon},
    main::MainView,
    theme::{ROW_HEIGHT, ROW_RADIUS, Theme},
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
    spot: Option<DropSpot>,
    expanding: Option<(FolderId, Task<()>)>,
}

impl SidebarDrop {
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

impl MainView {
    pub(super) fn drag_folder_over_row(
        &mut self,
        row: &FolderRow,
        event: &DragMoveEvent<DraggedFolder>,
        cx: &mut Context<'_, MainView>,
    ) {
        let bounds = event.bounds;
        let spot = if bounds.contains(&event.event.position) {
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
        cx: &mut Context<'_, MainView>,
    ) {
        let spot = if event.bounds.contains(&event.event.position) {
            Some(DropSpot::Into(row.id))
        } else {
            self.spot_leaving(row.id)
        };
        self.set_drop_spot(spot, row, cx);
    }

    /// The spot after the pointer left a row: cleared if it was that row's, kept if
    /// another row has already taken it.
    fn spot_leaving(&self, row: FolderId) -> Option<DropSpot> {
        match self.sidebar_drop.spot {
            Some(DropSpot::Into(id) | DropSpot::Before(id) | DropSpot::After(id)) if id == row => {
                None
            }
            spot => spot,
        }
    }

    pub(super) fn drag_folder_over_end(
        &mut self,
        event: &DragMoveEvent<DraggedFolder>,
        cx: &mut Context<'_, MainView>,
    ) {
        let spot = if event.bounds.contains(&event.event.position) {
            Some(DropSpot::End)
        } else if self.sidebar_drop.spot == Some(DropSpot::End) {
            None
        } else {
            return;
        };
        if self.sidebar_drop.spot != spot {
            self.sidebar_drop.spot = spot;
            cx.notify();
        }
    }

    fn set_drop_spot(
        &mut self,
        spot: Option<DropSpot>,
        row: &FolderRow,
        cx: &mut Context<'_, MainView>,
    ) {
        if self.sidebar_drop.spot == spot {
            return;
        }
        self.sidebar_drop.spot = spot;
        if spot == Some(DropSpot::Into(row.id)) && row.disclosure == Disclosure::Collapsed {
            self.expand_after_hover(row.id, cx);
        }
        cx.notify();
    }

    /// Opens a collapsed parent the drag has rested on, like Finder's spring-loading.
    fn expand_after_hover(&mut self, id: FolderId, cx: &mut Context<'_, MainView>) {
        if self
            .sidebar_drop
            .expanding
            .as_ref()
            .is_some_and(|(at, _)| *at == id)
        {
            return;
        }
        let task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(HOVER_TO_EXPAND).await;
            let _ = this.update(cx, |this, cx| {
                this.sidebar_drop.expanding = None;
                if cx.has_active_drag() && this.sidebar_drop.spot == Some(DropSpot::Into(id)) {
                    this.outline.expand(id, cx);
                    cx.notify();
                }
            });
        });
        self.sidebar_drop.expanding = Some((id, task));
    }

    pub(super) fn drop_folder(&mut self, folder: FolderId, cx: &mut Context<'_, MainView>) {
        let Some(spot) = self.sidebar_drop.spot.take() else {
            return;
        };
        let tree = self.library.read(cx).folder_tree();
        let Some((parent, before)) = placement(tree, folder, spot) else {
            return;
        };
        if let Some(parent) = parent {
            self.outline.expand(parent, cx);
        }
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| store.place_folder(folder, parent, before));
        });
    }

    pub(super) fn drop_bookmark(&mut self, bookmark: BookmarkId, cx: &mut Context<'_, MainView>) {
        if let Some(DropSpot::Into(folder)) = self.sidebar_drop.spot.take() {
            self.set_in_folder(bookmark, folder, true, cx);
        }
    }
}

/// What follows the pointer during a drag: the folder's mark and name, or the bookmark's
/// title, on a small chip.
pub(in crate::ui::main) struct DragPreview {
    label: SharedString,
    mark: PreviewMark,
}

enum PreviewMark {
    Folder(Option<FolderIcon>, knoa_core::FolderColor),
    Bookmark,
}

impl DragPreview {
    pub(super) fn folder(row: &FolderRow) -> DragPreview {
        DragPreview {
            label: row.name.clone(),
            mark: PreviewMark::Folder(row.icon.clone(), row.color),
        }
    }

    pub fn bookmark(dragged: &DraggedBookmark) -> DragPreview {
        DragPreview {
            label: dragged.title.clone(),
            mark: PreviewMark::Bookmark,
        }
    }

    /// The preview as an entity, which is what GPUI's `on_drag` asks for.
    pub fn build(self, cx: &mut gpui::App) -> gpui::Entity<DragPreview> {
        cx.new(|_| self)
    }
}

impl Render for DragPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let mark = match &self.mark {
            PreviewMark::Folder(glyph, color) => folder_mark(
                glyph.as_ref(),
                theme.folder_color(*color),
                px(15.),
                Finish::Line,
            )
            .into_any_element(),
            PreviewMark::Bookmark => icon("link", px(14.), theme.muted).into_any_element(),
        };
        div()
            .flex()
            .items_center()
            .gap_2()
            .h(ROW_HEIGHT)
            .max_w(px(240.))
            .px_2p5()
            .rounded(ROW_RADIUS)
            .bg(theme.panel)
            .border_1()
            .border_color(theme.hairline)
            .shadow_md()
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.text)
            .child(mark)
            .child(div().truncate().child(self.label.clone()))
    }
}
