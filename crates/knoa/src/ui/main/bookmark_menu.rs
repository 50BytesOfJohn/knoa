//! What can be done to one bookmark, as a menu. A card's right-click and the detail
//! panel's more button show the same one, so each action is found in the same place.
//!
//! Items act on the bookmark the menu was opened for. Right-clicking a card leaves the
//! selection alone, as menus do on macOS, so the shortcuts, which act on the selection,
//! are shown only when the menu is for the selected bookmark.

use gpui::{
    Action, App, Context, Div, FocusHandle, Hsla, InteractiveElement, IntoElement, ParentElement,
    Pixels, SharedString, Styled, WeakEntity, Window, div, px,
};
use gpui_component::{
    Side,
    menu::{ContextMenu, ContextMenuExt as _, PopupMenu, PopupMenuItem},
};
use knoa_browser::Activation;
use knoa_core::{Bookmark, BookmarkId, Folder, FolderIcon, FolderId, FolderTree, Place};

use super::{
    CopyLink, DeleteSelected, MainView, OpenSelected, OpenSelectedInBackground, QuickLook,
    ShowDetails, ToggleUpNext, picture::has_picture,
};
use crate::{
    i18n::t,
    services::thumbnails::{Refresh, Thumbnails},
    ui::{
        components::{Finish, folder_mark, icon_asset},
        theme::Theme,
    },
};

/// The folder submenu's fixed width, and the least the menu takes. The submenu picks the
/// side to open on by guessing its own width as the parent's, since it hasn't been drawn
/// yet; never being wider than the parent keeps it from running off the window.
const MENU_WIDTH: Pixels = px(240.);
/// Longer folder names are cut so the check after them stays inside the fixed width.
const FOLDER_NAME_CHARS: usize = 26;
/// How far a subfolder sits in from its parent, and the characters of name that costs.
const MENU_INDENT: Pixels = px(16.);
const INDENT_CHARS: usize = 3;
/// The size gpui-component gives a menu item's icon (`xsmall`).
const MENU_ICON: Pixels = px(12.);

/// Everything the menu shows, read once when it opens.
struct Target {
    bookmark: Bookmark,
    folders: FolderTree,
    place: Place,
    cards_focus: FocusHandle,
    refresh: Option<Refresh>,
    /// Whether the selection is this bookmark, so its shortcuts would act on it.
    selected: bool,
}

impl Target {
    fn read(view: &WeakEntity<MainView>, id: BookmarkId, cx: &App) -> Option<Target> {
        let view = view.upgrade()?.read(cx);
        let library = view.library.read(cx);
        Some(Target {
            bookmark: library.bookmark(id)?.clone(),
            folders: library.folder_tree().clone(),
            place: library.place(),
            cards_focus: view.cards_focus.clone(),
            refresh: Thumbnails::entity(cx).read(cx).refresh_of(id),
            selected: view.selected == Some(id),
        })
    }

    /// Shows `action`'s shortcut on the row, when the shortcut would act on this bookmark.
    fn shortcut(&self, row: PopupMenuItem, action: impl Action) -> PopupMenuItem {
        if self.selected {
            row.action(Box::new(action))
        } else {
            row
        }
    }

    fn is_in(&self, folder: FolderId) -> bool {
        self.bookmark
            .folders
            .iter()
            .any(|link| link.folder == folder)
    }

    /// The folder being viewed, when the bookmark is in it.
    fn viewed_folder(&self) -> Option<&Folder> {
        let Place::Folder(viewed) = self.place else {
            return None;
        };
        self.folders.get(viewed).filter(|f| self.is_in(f.id))
    }
}

/// Gives `element` the bookmark's menu on right-click.
pub(super) fn with_bookmark_menu<E>(
    element: E,
    id: BookmarkId,
    cx: &mut Context<'_, MainView>,
) -> ContextMenu<E>
where
    E: InteractiveElement + ParentElement + Styled + IntoElement + 'static,
{
    let view = cx.entity().downgrade();
    element.context_menu(move |menu, window, cx| bookmark_menu(menu, &view, id, window, cx))
}

/// The menu's items for bookmark `id`; empty if it is gone.
pub(super) fn bookmark_menu(
    menu: PopupMenu,
    view: &WeakEntity<MainView>,
    id: BookmarkId,
    window: &mut Window,
    cx: &mut Context<'_, PopupMenu>,
) -> PopupMenu {
    let Some(target) = Target::read(view, id, cx) else {
        return menu;
    };
    let menu = menu
        .action_context(target.cards_focus.clone())
        .min_w(MENU_WIDTH);
    if target.bookmark.deleted_at.is_some() {
        trash_items(menu, view, &target)
    } else {
        let menu = opening_items(menu, view, &target).separator();
        let menu = filing_items(menu, view, &target, window, cx).separator();
        let menu = refreshing_items(menu, view, &target).separator();
        menu.item(target.shortcut(
            item(
                view,
                t!("main-move-to-trash"),
                "trash",
                id,
                MainView::move_to_trash,
            ),
            DeleteSelected,
        ))
    }
}

fn opening_items(menu: PopupMenu, view: &WeakEntity<MainView>, target: &Target) -> PopupMenu {
    let id = target.bookmark.id;
    let open = item(
        view,
        t!("main-open"),
        "external-link",
        id,
        |this, id, cx| {
            this.open_bookmark(id, Activation::Foreground, cx);
        },
    );
    let open_behind = item(
        view,
        t!("main-open-in-background"),
        "app-window",
        id,
        |this, id, cx| {
            this.open_bookmark(id, Activation::Background, cx);
        },
    );
    let copy = item(
        view,
        t!("main-copy-link"),
        "link",
        id,
        MainView::copy_link_of,
    );
    let details = item(view, t!("main-show-details"), "info", id, MainView::select);
    let menu = menu
        .item(target.shortcut(open, OpenSelected))
        .item(target.shortcut(open_behind, OpenSelectedInBackground))
        .item(target.shortcut(copy, CopyLink))
        .item(target.shortcut(details, ShowDetails));
    if has_picture(&target.bookmark) {
        picture_items(menu, view, target)
    } else {
        menu
    }
}

/// Copy and save show no keys: theirs belong to the viewer, where ⌘C copies the picture.
fn picture_items(menu: PopupMenu, view: &WeakEntity<MainView>, target: &Target) -> PopupMenu {
    let id = target.bookmark.id;
    let view_picture = {
        let view = view.clone();
        PopupMenuItem::new(t!("main-view-picture"))
            .icon(icon_asset("maximize-2"))
            .on_click(move |_, window, cx| {
                let _ = view.update(cx, |this, cx| this.open_viewer(id, window, cx));
            })
    };
    menu.separator()
        .item(target.shortcut(view_picture, QuickLook))
        .item(item(
            view,
            t!("main-copy-image"),
            "copy",
            id,
            MainView::copy_image_of,
        ))
        .item(item(
            view,
            t!("main-save-image"),
            "download",
            id,
            MainView::save_image_of,
        ))
}

fn filing_items(
    menu: PopupMenu,
    view: &WeakEntity<MainView>,
    target: &Target,
    window: &mut Window,
    cx: &mut Context<'_, PopupMenu>,
) -> PopupMenu {
    let id = target.bookmark.id;
    let (label, icon) = if target.bookmark.up_next {
        (t!("main-remove-from-up-next"), "bookmark-minus")
    } else {
        (t!("main-add-to-up-next"), "bookmark-plus")
    };
    let up_next = item(view, label, icon, id, MainView::set_up_next);
    let menu = menu.item(target.shortcut(up_next, ToggleUpNext));
    let menu = folder_submenu(menu, view, target, window, cx);
    let Some(folder) = target.viewed_folder() else {
        return menu;
    };
    let folder_id = folder.id;
    menu.item(item(
        view,
        t!(
            "main-remove-from-folder",
            folder = (target.folders.path_name(folder_id))
        ),
        "folder-minus",
        id,
        move |this, id, cx| this.set_in_folder(id, folder_id, false, cx),
    ))
}

/// Greyed out with no folders yet rather than hidden, so the action is still discoverable.
fn folder_submenu(
    menu: PopupMenu,
    view: &WeakEntity<MainView>,
    target: &Target,
    window: &mut Window,
    cx: &mut Context<'_, PopupMenu>,
) -> PopupMenu {
    if target.folders.is_empty() {
        return menu.item(
            PopupMenuItem::new(t!("main-add-to-folder"))
                .icon(icon_asset("folder-plus"))
                .disabled(true),
        );
    }
    let (view, id) = (view.clone(), target.bookmark.id);
    let folders: Vec<(Folder, usize, bool)> = target
        .folders
        .folders()
        .iter()
        .map(|f| (f.clone(), target.folders.depth(f.id), target.is_in(f.id)))
        .collect();
    menu.submenu_with_icon(
        Some(icon_asset("folder-plus")),
        t!("main-add-to-folder"),
        window,
        cx,
        move |menu, _, cx| folder_items(menu, &view, id, &folders, cx),
    )
}

/// One row per folder, tinted and indented like the sidebar's. Checked ones hold the
/// bookmark, and choosing one again takes it out. The check sits on the right because a
/// left check would replace the folder's icon.
fn folder_items(
    menu: PopupMenu,
    view: &WeakEntity<MainView>,
    id: BookmarkId,
    folders: &[(Folder, usize, bool)],
    cx: &App,
) -> PopupMenu {
    let theme = Theme::get(cx);
    let menu = menu
        .min_w(MENU_WIDTH)
        .max_w(MENU_WIDTH)
        .check_side(Side::Right);
    folders.iter().fold(menu, |menu, (folder, depth, filed)| {
        let (folder_id, depth, filed) = (folder.id, *depth, *filed);
        let view = view.clone();
        let tint = theme.folder_color(folder.color);
        let chars = FOLDER_NAME_CHARS - INDENT_CHARS * depth;
        let (icon, label) = (folder.icon.clone(), shorten(&folder.name, chars));
        menu.item(
            PopupMenuItem::element(move |_, _| {
                folder_row(icon.as_ref(), tint, &label).pl(MENU_INDENT * depth as f32)
            })
            .checked(filed)
            .on_click(move |_, _, cx| {
                let _ = view.update(cx, |this, cx| {
                    this.set_in_folder(id, folder_id, !filed, cx);
                });
            }),
        )
    })
}

/// The folder's mark and name laid out as a menu item lays out its icon and label, drawn
/// here because a menu icon can only be an SVG and the mark can be an emoji.
fn folder_row(icon: Option<&FolderIcon>, tint: Hsla, label: &str) -> Div {
    div()
        .flex()
        .items_center()
        .gap_x_1()
        .child(folder_mark(icon, tint, MENU_ICON, Finish::Line))
        .child(div().flex_1().truncate().child(label.to_owned()))
}

fn shorten(name: &str, chars: usize) -> String {
    if name.chars().count() <= chars {
        return name.to_owned();
    }
    let kept: String = name.chars().take(chars - 1).collect();
    format!("{}…", kept.trim_end())
}

fn refreshing_items(menu: PopupMenu, view: &WeakEntity<MainView>, target: &Target) -> PopupMenu {
    let id = target.bookmark.id;
    let refreshing = target.refresh == Some(Refresh::Running);
    let thumbnail = if refreshing {
        t!("main-refreshing-thumbnail")
    } else {
        t!("main-refresh-thumbnail")
    };
    let organizing = target.bookmark.status.is_waiting();
    let organize = if organizing {
        t!("main-organizing")
    } else if target.bookmark.status.is_organized() {
        t!("main-organize-again")
    } else {
        t!("main-organize")
    };
    menu.item(item(view, thumbnail, "image", id, MainView::refresh_thumbnail).disabled(refreshing))
        .item(item(view, organize, "sparkles", id, MainView::organize).disabled(organizing))
}

fn trash_items(menu: PopupMenu, view: &WeakEntity<MainView>, target: &Target) -> PopupMenu {
    let id = target.bookmark.id;
    opening_items(menu, view, target)
        .separator()
        .item(item(
            view,
            t!("main-restore"),
            "undo-2",
            id,
            MainView::restore,
        ))
        .item(item(
            view,
            t!("main-delete-now"),
            "trash",
            id,
            MainView::delete_for_good,
        ))
}

/// A row that runs `act` on the main view for bookmark `id`.
fn item(
    view: &WeakEntity<MainView>,
    label: impl Into<SharedString>,
    icon: &str,
    id: BookmarkId,
    act: impl Fn(&mut MainView, BookmarkId, &mut Context<'_, MainView>) + 'static,
) -> PopupMenuItem {
    let view = view.clone();
    PopupMenuItem::new(label)
        .icon(icon_asset(icon))
        .on_click(move |_, _, cx| {
            let _ = view.update(cx, |this, cx| act(this, id, cx));
        })
}
