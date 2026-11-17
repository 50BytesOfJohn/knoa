//! One folder's row: indented under its parent, its icon giving way to a disclosure
//! chevron and its count to a ⋯ menu button under the pointer, with drop marks while
//! dragging.
//!
//! The swaps are styles keyed on the row's hover group, so hovering needs no state: GPUI
//! repaints the row as the pointer enters and leaves it.

use std::rc::Rc;

use gpui::{
    Animation, AnimationExt as _, AnyElement, Context, DragMoveEvent, InteractiveElement as _,
    IntoElement, ParentElement as _, Pixels, Radians, StatefulInteractiveElement as _, Styled as _,
    Transformation, div, ease_out_quint, prelude::FluentBuilder as _, px,
};
use gpui_component::menu::{ContextMenuExt as _, DropdownMenu as _};
use knoa_core::FolderId;

use super::{
    SidebarEvent, SidebarView,
    drag_preview::DragPreview,
    drop::{DraggedBookmark, DraggedFolder, DropSpot},
    folder_menu::folder_menu,
    folders::drop_line,
    outline::{Disclosure, FolderRow},
    row::{ICON, PADDING_LEFT, ROW_GAP, count, label, row_frame},
};
use crate::{
    assets::line_icon,
    ui::{
        components::{Finish, folder_mark, svg_icon},
        main::menu_trigger::MenuTrigger,
        motion,
        theme::Theme,
    },
};

/// Each level sits this far in from its parent, so a child's icon is under its parent's
/// label.
const INDENT: Pixels = px(26.);
/// The chevron and ⋯ buttons, centred where the icon and the count are.
const ROW_BUTTON: Pixels = px(24.);
const ROW_BUTTON_ICON: Pixels = px(14.);
const ROW_BUTTON_TOP: Pixels = px(3.);
const MENU_BUTTON_RIGHT: Pixels = px(5.);
/// Hover group of the row under the pointer; GPUI matches the nearest one, so every row
/// can share the name.
const ROW_GROUP: &str = "folder-row";

impl SidebarView {
    pub(super) fn folder_row(
        &self,
        row: &Rc<FolderRow>,
        selected: bool,
        spot: Option<DropSpot>,
        cx: &mut Context<'_, SidebarView>,
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
            // The gap after the row belongs to its drop area, so a drag never falls between.
            .pb(ROW_GAP)
            .child(self.folder_row_body(row, selected, target, indent, cx))
            .when_some(target, |s, spot| {
                s.child(drop_line(spot, PADDING_LEFT + indent, accent))
            })
            .on_click(cx.listener(move |_, _, _, cx| {
                cx.emit(SidebarEvent::Show(knoa_core::Place::Folder(id)));
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

    fn folder_row_body(
        &self,
        row: &FolderRow,
        selected: bool,
        target: Option<DropSpot>,
        indent: Pixels,
        cx: &mut Context<'_, SidebarView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx);
        let finish = if selected {
            Finish::Tinted
        } else {
            Finish::Line
        };
        let receiving = matches!(target, Some(DropSpot::Into(_)));
        let parent = row.disclosure != Disclosure::Leaf;
        let menu_open = self.folder_menu == Some(row.id);
        let mark = folder_mark(
            row.icon.as_ref(),
            theme.folder_color(row.color),
            ICON,
            finish,
        )
        .when(parent, |s| s.group_hover(ROW_GROUP, |s| s.opacity(0.)));
        let count = count(row.count.clone(), theme)
            .when(menu_open, |s| s.opacity(0.))
            .group_hover(ROW_GROUP, |s| s.opacity(0.));
        row_frame(selected, theme)
            .group(ROW_GROUP)
            .pl(PADDING_LEFT + indent)
            .when(receiving, |s| s.bg(theme.accent.opacity(0.18)))
            .child(mark)
            .child(label(row.name.clone()))
            .child(count)
            .when(parent, |s| s.child(self.disclosure(row, indent, cx)))
            .child(self.menu_button(row.id, menu_open, cx))
    }

    /// The chevron over a parent's icon, turned down while it is open.
    fn disclosure(
        &self,
        row: &FolderRow,
        indent: Pixels,
        cx: &mut Context<'_, SidebarView>,
    ) -> AnyElement {
        let theme = Theme::get(cx);
        let id = row.id;
        let open = row.disclosure == Disclosure::Expanded;
        // A new id whenever it opens or closes replays the turn from where it was.
        let turn_id = (
            if open {
                "chevron-open"
            } else {
                "chevron-closed"
            },
            id.0 as usize,
        );
        let (from, to) = if open { (0., 90.) } else { (90., 0.) };
        let chevron = svg_icon(line_icon!("chevron-right"), ROW_BUTTON_ICON, theme.muted)
            .with_animation(
                turn_id,
                Animation::new(motion::CHEVRON).with_easing(ease_out_quint()),
                move |svg, delta| {
                    let degrees: f32 = from + (to - from) * delta;
                    svg.with_transformation(Transformation::rotate(Radians(degrees.to_radians())))
                },
            );
        row_button(("disclosure", id.0 as usize), theme)
            .left(PADDING_LEFT + indent - (ROW_BUTTON - ICON) / 2.)
            .child(chevron)
            .on_click(cx.listener(move |this, _, _, cx| {
                cx.stop_propagation();
                this.toggle_folder(id, cx);
            }))
            .into_any_element()
    }

    /// The ⋯ over the count, which opens the same menu as a right click.
    fn menu_button(
        &self,
        id: FolderId,
        open: bool,
        cx: &mut Context<'_, SidebarView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx);
        let base = row_button(("folder-more", id.0 as usize), theme)
            .right(MENU_BUTTON_RIGHT)
            .when(open, gpui::Styled::flex)
            .child(svg_icon(
                line_icon!("ellipsis"),
                ROW_BUTTON_ICON,
                theme.muted,
            ));
        let view = cx.entity().downgrade();
        let tracked = view.clone();
        MenuTrigger::new(base, theme.hover)
            .dropdown_menu(move |menu, window, cx| folder_menu(menu, &view, id, window, cx))
            .on_open_change(move |open, _, cx| {
                let _ = tracked.update(cx, |this, cx| {
                    this.folder_menu = open.then_some(id);
                    cx.notify();
                });
            })
    }
}

/// A small button over the row, shown while the pointer is on it.
fn row_button(id: (&'static str, usize), theme: &Theme) -> gpui::Stateful<gpui::Div> {
    let (hover, text) = (theme.hover, theme.text);
    div()
        .id(id)
        .absolute()
        .top(ROW_BUTTON_TOP)
        .size(ROW_BUTTON)
        .rounded(px(6.))
        .hidden()
        .group_hover(ROW_GROUP, gpui::Styled::flex)
        .items_center()
        .justify_center()
        .text_color(theme.muted)
        .hover(move |s| s.bg(hover).text_color(text))
        // A click on the button isn't a click on the row.
        .on_click(|_, _, cx| cx.stop_propagation())
}
