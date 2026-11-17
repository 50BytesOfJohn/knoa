//! The sidebar's folders: the section header with its new-folder button, the rows, and the
//! room after them that drops a folder at the end of the top level.

use gpui::{
    Context, DragMoveEvent, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    Pixels, StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};
use knoa_core::{FolderId, Place};

use super::{
    SIDEBAR_GROUP, SidebarEvent, SidebarView,
    drop::{DraggedFolder, DropSpot},
    row::PADDING_LEFT,
    titlebar::icon_button,
};
use crate::{
    assets::line_icon,
    ui::{components::tip, main::NewFolder, theme::Theme},
};

const HEADER_HEIGHT: Pixels = px(28.);
const HEADER_SPACE: Pixels = px(18.);
const ADD_BUTTON: Pixels = px(22.);
const ADD_ICON: Pixels = px(14.);
/// Room after the last folder to drop a folder at the end of the top level.
const END_DROP_ROOM: Pixels = px(28.);
const DROP_LINE: Pixels = px(2.);

impl SidebarView {
    /// "Folders", with a new folder button that shows while the pointer is over the
    /// sidebar.
    pub(super) fn folder_header(&self, cx: &mut Context<'_, SidebarView>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let add = icon_button(
            "new-folder",
            line_icon!("plus"),
            ADD_BUTTON,
            ADD_ICON,
            theme,
        )
        .rounded(px(6.))
        .hidden()
        .group_hover(SIDEBAR_GROUP, gpui::Styled::flex)
        .tooltip(tip("New folder", Some(&NewFolder), theme))
        .on_click(cx.listener(|_, _, _, cx| cx.emit(SidebarEvent::NewFolder(None))));
        div()
            .mt(HEADER_SPACE)
            .h(HEADER_HEIGHT)
            .flex()
            .items_center()
            .pl(px(8.))
            .pr(px(5.))
            .text_size(px(11.5))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme.muted)
            .child(div().flex_1().child("Folders"))
            .child(add)
    }

    pub(super) fn folder_list(
        &mut self,
        current: Place,
        cx: &mut Context<'_, SidebarView>,
    ) -> gpui::Div {
        if !cx.has_active_drag() {
            self.drop.end();
        }
        let library = self.library.read(cx);
        let rows = self
            .outline
            .rows(library.folder_tree(), library.folders_generation());
        let spot = self.drop.spot();
        let mut list = div().flex().flex_col();
        for row in rows.iter() {
            let selected = current == Place::Folder(row.id);
            list = list.child(self.folder_row(row, selected, spot, cx));
        }
        list.child(self.end_drop_room(spot == Some(DropSpot::End), cx))
    }

    /// Empty room after the last folder: a folder dropped here goes last at the top level.
    fn end_drop_room(
        &self,
        receiving: bool,
        cx: &mut Context<'_, SidebarView>,
    ) -> impl IntoElement {
        let accent = Theme::get(cx).accent;
        div()
            .id("folders-end")
            .relative()
            .h(END_DROP_ROOM)
            .when(receiving, |s| {
                s.child(drop_line(DropSpot::End, PADDING_LEFT, accent))
            })
            .on_drag_move(
                cx.listener(|this, event: &DragMoveEvent<DraggedFolder>, _, cx| {
                    this.drag_folder_over_end(event, cx);
                }),
            )
            .on_drop(cx.listener(|this, dragged: &DraggedFolder, _, cx| {
                this.drop_folder(dragged.0.id, cx);
            }))
    }

    pub(super) fn toggle_folder(&mut self, id: FolderId, cx: &mut Context<'_, SidebarView>) {
        self.outline.toggle(id, cx);
        cx.notify();
    }

    /// Moves a folder last under `parent`, which opens to show it.
    pub(super) fn move_folder(
        &mut self,
        id: FolderId,
        parent: Option<FolderId>,
        cx: &mut Context<'_, SidebarView>,
    ) {
        if let Some(parent) = parent {
            self.expand(parent, cx);
        }
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| store.move_folder(id, parent));
        });
    }
}

/// The line that shows where a dragged folder would land between rows, indented to the
/// level it would take. Each row's drop area takes in the gap after it, so a line after
/// one row and before the next fall on the same spot.
pub(super) fn drop_line(spot: DropSpot, left: Pixels, color: gpui::Hsla) -> gpui::Div {
    let line = div()
        .absolute()
        .left(left)
        .right_0()
        .h(DROP_LINE)
        .rounded_full()
        .bg(color);
    let offset = -DROP_LINE / 2.;
    match spot {
        DropSpot::Before(_) | DropSpot::End => line.top(offset),
        DropSpot::After(_) => line.bottom(offset),
        DropSpot::Into(_) => div(),
    }
}
