//! What can be done to a folder, as its right-click menu.

use gpui::{App, Context, SharedString, WeakEntity, Window};
use gpui_component::menu::{PopupMenu, PopupMenuItem};
use knoa_core::{FolderId, FolderTree};

use super::{SidebarEvent, SidebarView};
use crate::ui::components::icon_asset;

/// Where a folder can move, read once when its menu opens.
struct MoveTargets {
    /// Shown when the folder is nested and could go to the top level.
    top_level: bool,
    /// Every valid new parent, by path.
    parents: Vec<(FolderId, SharedString)>,
}

impl MoveTargets {
    fn of(tree: &FolderTree, id: FolderId) -> MoveTargets {
        let current = tree.parent(id).map(|f| f.id);
        let parents = tree
            .folders()
            .iter()
            .filter(|f| Some(f.id) != current && tree.check_move(id, Some(f.id)).is_ok())
            .map(|f| (f.id, SharedString::from(tree.path_name(f.id))))
            .collect();
        MoveTargets {
            top_level: current.is_some(),
            parents,
        }
    }

    fn is_empty(&self) -> bool {
        !self.top_level && self.parents.is_empty()
    }
}

pub(super) fn folder_menu(
    menu: PopupMenu,
    view: &WeakEntity<SidebarView>,
    id: FolderId,
    window: &mut Window,
    cx: &mut Context<'_, PopupMenu>,
) -> PopupMenu {
    let Some(tree) = read_tree(view, cx) else {
        return menu;
    };
    if tree.get(id).is_none() {
        return menu;
    }
    let can_nest = tree.check_new_child(Some(id)).is_ok();
    let targets = MoveTargets::of(&tree, id);
    let menu = if can_nest {
        menu.item(item(view, "New subfolder", "folder-plus", move |_, cx| {
            cx.emit(SidebarEvent::NewFolder(Some(id)));
        }))
    } else {
        menu
    };
    let menu = menu.item(item(view, "Edit folder…", "pencil", move |_, cx| {
        cx.emit(SidebarEvent::EditFolder(id));
    }));
    let menu = move_submenu(menu, view, id, targets, window, cx);
    menu.item(item(view, "Delete…", "trash", move |_, cx| {
        cx.emit(SidebarEvent::DeleteFolder(id));
    }))
    .separator()
    .item(item(view, "New folder", "plus", |_, cx| {
        cx.emit(SidebarEvent::NewFolder(None));
    }))
}

fn read_tree(view: &WeakEntity<SidebarView>, cx: &App) -> Option<FolderTree> {
    let view = view.upgrade()?;
    Some(view.read(cx).library.read(cx).folder_tree().clone())
}

fn move_submenu(
    menu: PopupMenu,
    view: &WeakEntity<SidebarView>,
    id: FolderId,
    targets: MoveTargets,
    window: &mut Window,
    cx: &mut Context<'_, PopupMenu>,
) -> PopupMenu {
    if targets.is_empty() {
        return menu.item(
            PopupMenuItem::new("Move to")
                .icon(icon_asset("folder-input"))
                .disabled(true),
        );
    }
    let view = view.clone();
    menu.submenu_with_icon(
        Some(icon_asset("folder-input")),
        "Move to",
        window,
        cx,
        move |menu, _, _| {
            let menu = if targets.top_level {
                let top = item(&view, "Top level", "folder-up", move |this, cx| {
                    this.move_folder(id, None, cx);
                });
                let menu = menu.item(top);
                if targets.parents.is_empty() {
                    menu
                } else {
                    menu.separator()
                }
            } else {
                menu
            };
            targets.parents.iter().fold(menu, |menu, (parent, path)| {
                let parent = *parent;
                menu.item(item(&view, path.clone(), "folder", move |this, cx| {
                    this.move_folder(id, Some(parent), cx);
                }))
            })
        },
    )
}

/// A row that runs `act` on the sidebar.
fn item(
    view: &WeakEntity<SidebarView>,
    label: impl Into<SharedString>,
    icon: &str,
    act: impl Fn(&mut SidebarView, &mut Context<'_, SidebarView>) + 'static,
) -> PopupMenuItem {
    let view = view.clone();
    PopupMenuItem::new(label)
        .icon(icon_asset(icon))
        .on_click(move |_, _, cx| {
            let _ = view.update(cx, |this, cx| act(this, cx));
        })
}
