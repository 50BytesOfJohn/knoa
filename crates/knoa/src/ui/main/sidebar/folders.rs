//! The sidebar's folder rows: indented under their parents, with a disclosure chevron on
//! parents, ⌘1 to ⌘9 on the first visible rows, and drop marks while dragging.

use std::rc::Rc;

use gpui::{
    Context, DragMoveEvent, InteractiveElement as _, IntoElement, ParentElement as _, Pixels,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};
use gpui_component::menu::ContextMenuExt as _;
use knoa_core::{FolderId, Place};

use super::{
    ROW_INSET, ROW_PADDING,
    drop::{DragPreview, DraggedBookmark, DraggedFolder, DropSpot},
    folder_menu::folder_menu,
    outline::{Disclosure, FolderRow},
    row_frame,
};
use crate::ui::{
    components::{Finish, folder_mark, icon},
    main::MainView,
    theme::{ROW_HEIGHT, ROW_RADIUS, Theme},
};

/// How far each level of folders sits in from its parent: one mark and its gap, so a
/// child's mark starts under its parent's name.
const INDENT: Pixels = px(23.);
/// The disclosure chevron's hit area, kept on every row of a nested tree so counts line up.
const DISCLOSURE: Pixels = px(16.);
/// Fixed trailing columns in nested libraries, including rows without a hint or count.
const SHORTCUT_WIDTH: Pixels = px(24.);
const COUNT_WIDTH: Pixels = px(24.);
const DROP_LINE: Pixels = px(2.);
/// Room under "New folder" to drop a folder at the end of the top level.
const END_DROP_ROOM: Pixels = px(28.);
const SHORTCUTS: [&str; 9] = ["⌘1", "⌘2", "⌘3", "⌘4", "⌘5", "⌘6", "⌘7", "⌘8", "⌘9"];

impl MainView {
    pub(super) fn folder_list(
        &mut self,
        current: Place,
        cx: &mut Context<'_, MainView>,
    ) -> gpui::Div {
        let dragging = cx.has_active_drag();
        if !dragging {
            self.sidebar_drop.end();
        }
        let library = self.library.read(cx);
        let rows = self
            .outline
            .rows(library.folder_tree(), library.folders_generation());
        let nested = self.outline.is_nested();
        let spot = self.sidebar_drop.spot();
        let mut list = div().flex().flex_col().px(ROW_INSET);
        for (index, row) in rows.iter().enumerate() {
            let selected = current == Place::Folder(row.id);
            let shortcut = SHORTCUTS.get(index).copied();
            list = list.child(self.folder_row(row, selected, shortcut, nested, spot, cx));
        }
        list.child(self.new_folder_row(spot == Some(DropSpot::End), cx))
    }

    fn folder_row(
        &self,
        row: &Rc<FolderRow>,
        selected: bool,
        shortcut: Option<&'static str>,
        nested: bool,
        spot: Option<DropSpot>,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let accent = Theme::get(cx).accent;
        let id = row.id;
        let target = spot.filter(|spot| match spot {
            DropSpot::Into(at) | DropSpot::Before(at) | DropSpot::After(at) => *at == id,
            DropSpot::End => false,
        });
        let indent = INDENT * row.depth as f32;
        let (over_row, under_row) = (Rc::clone(row), Rc::clone(row));
        let view = cx.entity().downgrade();
        div()
            .id(("folder-row", id.0 as usize))
            .relative()
            // Keep the places' spacing inside the drop area, with no gap before the first row.
            .pb_0p5()
            .child(self.folder_row_body(row, selected, shortcut, nested, target, indent, cx))
            .when_some(target, |s, spot| {
                s.child(drop_line(spot, ROW_PADDING + indent, accent))
            })
            .on_click(cx.listener(move |this, _, window, cx| {
                this.set_place(Place::Folder(id), window, cx);
            }))
            .on_drag(DraggedFolder(Rc::clone(row)), |dragged, _, _, cx| {
                DragPreview::folder(&dragged.0).build(cx)
            })
            .on_drag_move(
                cx.listener(move |this, event: &DragMoveEvent<DraggedFolder>, _, cx| {
                    this.drag_folder_over_row(&over_row, event, cx);
                }),
            )
            .on_drag_move(cx.listener(
                move |this, event: &DragMoveEvent<DraggedBookmark>, _, cx| {
                    this.drag_bookmark_over_row(&under_row, event, cx);
                },
            ))
            .on_drop(cx.listener(|this, dragged: &DraggedFolder, _, cx| {
                this.drop_folder(dragged.0.id, cx);
            }))
            .on_drop(cx.listener(|this, dragged: &DraggedBookmark, _, cx| {
                this.drop_bookmark(dragged.id, cx);
            }))
            .context_menu(move |menu, window, cx| folder_menu(menu, &view, id, window, cx))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "one row's look, each argument a separate fact about it"
    )]
    fn folder_row_body(
        &self,
        row: &FolderRow,
        selected: bool,
        shortcut: Option<&'static str>,
        nested: bool,
        target: Option<DropSpot>,
        indent: Pixels,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx);
        let color = theme.folder_color(row.color);
        let finish = if selected {
            Finish::Tinted
        } else {
            Finish::Line
        };
        let receiving = matches!(target, Some(DropSpot::Into(_)));
        row_frame(selected, theme)
            .pl(ROW_PADDING + indent)
            .pr(if nested { px(2.) } else { ROW_PADDING })
            .hover(|s| s.bg(theme.hover))
            .when(receiving, |s| s.bg(theme.accent.opacity(0.18)))
            .child(folder_mark(row.icon.as_ref(), color, px(15.), finish))
            .child(div().flex_1().truncate().child(row.name.clone()))
            .when(nested || shortcut.is_some(), |s| {
                s.child(
                    div()
                        .when(nested, |s| s.w(SHORTCUT_WIDTH).flex_none().text_right())
                        .text_xs()
                        .text_color(theme.faint)
                        .children(shortcut),
                )
            })
            .when(nested || !row.count.is_empty(), |s| {
                s.child(
                    div()
                        .when(nested, |s| s.w(COUNT_WIDTH).flex_none().text_right())
                        .text_xs()
                        .text_color(theme.muted)
                        .child(row.count.clone()),
                )
            })
            .when(nested, |s| s.child(self.disclosure(row, cx)))
    }

    /// The chevron that opens and closes a parent, or the empty space that keeps a
    /// leaf's count in line with its parent's.
    fn disclosure(&self, row: &FolderRow, cx: &mut Context<'_, MainView>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let id = row.id;
        let chevron = match row.disclosure {
            Disclosure::Leaf => None,
            Disclosure::Expanded => Some("chevron-down"),
            Disclosure::Collapsed => Some("chevron-right"),
        };
        div()
            .id(("disclosure", id.0 as usize))
            .size(DISCLOSURE)
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .rounded(ROW_RADIUS)
            .when_some(chevron, |s, name| {
                s.hover(|s| s.bg(theme.soft))
                    .child(icon(name, px(12.), theme.faint))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        cx.stop_propagation();
                        this.toggle_folder(id, cx);
                    }))
            })
    }

    /// "New folder", with room under it that drops a dragged folder at the end of the
    /// top level.
    fn new_folder_row(&self, receiving: bool, cx: &mut Context<'_, MainView>) -> impl IntoElement {
        let theme = Theme::get(cx);
        div()
            .id("folders-end")
            .relative()
            .pb(END_DROP_ROOM)
            .when(receiving, |s| {
                s.child(drop_line(DropSpot::End, ROW_PADDING, theme.accent))
            })
            .on_drag_move(
                cx.listener(|this, event: &DragMoveEvent<DraggedFolder>, _, cx| {
                    this.drag_folder_over_end(event, cx);
                }),
            )
            .on_drop(cx.listener(|this, dragged: &DraggedFolder, _, cx| {
                this.drop_folder(dragged.0.id, cx);
            }))
            .child(
                div()
                    .id("new-folder")
                    .flex()
                    .items_center()
                    .gap_2()
                    .h(ROW_HEIGHT)
                    .px(ROW_PADDING)
                    .rounded(ROW_RADIUS)
                    .text_sm()
                    .text_color(theme.muted)
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.hover))
                    .child(icon("plus", px(15.), theme.muted))
                    .child("New folder")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.new_folder(None, window, cx);
                    })),
            )
    }

    pub(in crate::ui::main) fn toggle_folder(
        &mut self,
        id: FolderId,
        cx: &mut Context<'_, MainView>,
    ) {
        self.outline.toggle(id, cx);
        cx.notify();
    }

    /// Moves a folder last under `parent`, which opens to show it.
    pub(in crate::ui::main) fn move_folder(
        &mut self,
        id: FolderId,
        parent: Option<FolderId>,
        cx: &mut Context<'_, MainView>,
    ) {
        if let Some(parent) = parent {
            self.outline.expand(parent, cx);
        }
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| store.move_folder(id, parent));
        });
    }
}

/// The line that shows where a dragged folder would land between rows, indented to the
/// level it would take.
fn drop_line(spot: DropSpot, left: Pixels, color: gpui::Hsla) -> gpui::Div {
    let line = div()
        .absolute()
        .left(left)
        .right_0()
        .h(DROP_LINE)
        .rounded_full()
        .bg(color);
    match spot {
        DropSpot::Before(_) | DropSpot::End => line.top(-DROP_LINE / 2.),
        DropSpot::After(_) => line.bottom(-DROP_LINE / 2.),
        DropSpot::Into(_) => div(),
    }
}
