//! The folder editor's parent: a menu of every folder this one can go under, by path, and
//! "None" for the top level.

use gpui::{
    Context, Div, InteractiveElement as _, ParentElement as _, SharedString, Styled as _,
    WeakEntity, div, px,
};
use gpui_component::{
    Side,
    menu::{DropdownMenu as _, PopupMenuItem},
};
use knoa_core::{FolderId, FolderTree};

use super::FolderEditor;
use crate::ui::{
    components::icon,
    main::menu_trigger::MenuTrigger,
    theme::{ROW_RADIUS, Theme},
};

const NONE: &str = "None";
/// Enough for every path at the deepest level the menu offers.
const MENU_WIDTH: f32 = 300.;
const MENU_MAX_HEIGHT: f32 = 320.;

impl FolderEditor {
    /// `None` when the folder has nowhere to go but the top level, which keeps the
    /// editor as it was for anyone with a single folder.
    pub(super) fn parent_picker(
        &self,
        theme: &Theme,
        cx: &mut Context<'_, FolderEditor>,
    ) -> Option<Div> {
        let tree = self.library.read(cx).folder_tree();
        let parents = self.valid_parents(tree);
        if parents.is_empty() && self.parent.is_none() {
            return None;
        }
        let label: SharedString = match self.parent {
            Some(id) if tree.get(id).is_some() => tree.path_name(id).into(),
            _ => NONE.into(),
        };
        let (current, editor) = (self.parent, cx.entity().downgrade());
        let field = div()
            .id("folder-parent")
            .flex_1()
            .min_w_0()
            .h(px(30.))
            .px_2p5()
            .flex()
            .items_center()
            .gap_2()
            .rounded(ROW_RADIUS)
            .border_1()
            .border_color(theme.hairline)
            .text_sm()
            .cursor_pointer()
            .child(div().flex_1().truncate().child(label))
            .child(icon("chevrons-up-down", px(13.), theme.muted));
        let picker = MenuTrigger::new(field, theme.soft).dropdown_menu(move |menu, _, _| {
            let menu = menu
                .min_w(px(MENU_WIDTH))
                .max_h(px(MENU_MAX_HEIGHT))
                .scrollable(true)
                .check_side(Side::Right)
                .item(choice(&editor, NONE.into(), None, current));
            parents.iter().fold(menu, |menu, (id, path)| {
                menu.item(choice(&editor, path.clone(), Some(*id), current))
            })
        });
        Some(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(div().text_sm().child("Parent"))
                .child(picker),
        )
    }

    /// Every folder this one may go under, by path, in sidebar order.
    fn valid_parents(&self, tree: &FolderTree) -> Vec<(FolderId, SharedString)> {
        let fits = |parent: FolderId| match &self.existing {
            Some(folder) => tree.check_move(folder.id, Some(parent)).is_ok(),
            None => tree.check_new_child(Some(parent)).is_ok(),
        };
        tree.folders()
            .iter()
            .filter(|f| fits(f.id))
            .map(|f| (f.id, SharedString::from(tree.path_name(f.id))))
            .collect()
    }
}

fn choice(
    editor: &WeakEntity<FolderEditor>,
    label: SharedString,
    parent: Option<FolderId>,
    current: Option<FolderId>,
) -> PopupMenuItem {
    let editor = editor.clone();
    PopupMenuItem::new(label)
        .checked(parent == current)
        .on_click(move |_, _, cx| {
            let _ = editor.update(cx, |editor, cx| {
                editor.parent = parent;
                cx.notify();
            });
        })
}
