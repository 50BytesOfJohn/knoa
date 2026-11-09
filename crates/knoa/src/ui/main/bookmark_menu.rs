//! What can be done to one bookmark, as a menu. A card's right-click and the detail
//! panel's more button show the same one, so each action is found in the same place.
//!
//! Items act on the bookmark the menu was opened for. Right-clicking a card leaves the
//! selection alone, as menus do on macOS, so the shortcuts, which act on the selection,
//! are shown only when the menu is for the selected bookmark.

use gpui::{
    Action, App, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Pixels,
    SharedString, Styled, WeakEntity, Window, px,
};
use gpui_component::{
    Icon, Side,
    menu::{ContextMenu, ContextMenuExt as _, PopupMenu, PopupMenuItem},
};
use knoa_browser::Activation;
use knoa_core::{Bookmark, BookmarkId, Folder, FolderId, Place, Status};

use super::{
    CopyLink, DeleteSelected, MainView, OpenSelected, OpenSelectedInBackground, ToggleUpNext,
};
use crate::{
    services::{
        organizer::Organizer,
        thumbnails::{Refresh, Thumbnails},
    },
    ui::theme::Theme,
};

/// The folder submenu's fixed width, and the least the menu takes. The submenu picks the
/// side to open on by guessing its own width as the parent's, since it hasn't been drawn
/// yet; never being wider than the parent keeps it from running off the window.
const MENU_WIDTH: Pixels = px(240.);
/// Longer folder names are cut so the check after them stays inside the fixed width.
const FOLDER_NAME_CHARS: usize = 26;

/// Everything the menu shows, read once when it opens.
struct Target {
    bookmark: Bookmark,
    folders: Vec<Folder>,
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
            folders: library.folders().to_vec(),
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
        self.folders
            .iter()
            .find(|f| f.id == viewed)
            .filter(|f| self.is_in(f.id))
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
            item(view, "Move to Trash", "trash", id, MainView::move_to_trash),
            DeleteSelected,
        ))
    }
}

fn opening_items(menu: PopupMenu, view: &WeakEntity<MainView>, target: &Target) -> PopupMenu {
    let id = target.bookmark.id;
    let open = item(view, "Open", "external-link", id, |this, id, cx| {
        this.open_bookmark(id, Activation::Foreground, cx);
    });
    let open_behind = item(
        view,
        "Open in background",
        "app-window",
        id,
        |this, id, cx| {
            this.open_bookmark(id, Activation::Background, cx);
        },
    );
    let copy = item(view, "Copy link", "link", id, |this, id, cx| {
        if let Some(url) = this.library.read(cx).bookmark(id).map(|b| b.url.clone()) {
            this.copy_to_clipboard(id, url, cx);
        }
    });
    menu.item(target.shortcut(open, OpenSelected))
        .item(target.shortcut(open_behind, OpenSelectedInBackground))
        .item(target.shortcut(copy, CopyLink))
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
        ("Remove from Up next", "bookmark-minus")
    } else {
        ("Add to Up next", "bookmark-plus")
    };
    let up_next = item(view, label, icon, id, MainView::set_up_next);
    let menu = menu.item(target.shortcut(up_next, ToggleUpNext));
    let menu = if target.folders.is_empty() {
        // Greyed out rather than hidden, so the action is still discoverable.
        menu.item(
            PopupMenuItem::new("Add to folder")
                .icon(menu_icon("folder-plus"))
                .disabled(true),
        )
    } else {
        let view = view.clone();
        let folders: Vec<(Folder, bool)> = target
            .folders
            .iter()
            .map(|f| (f.clone(), target.is_in(f.id)))
            .collect();
        menu.submenu_with_icon(
            Some(menu_icon("folder-plus")),
            "Add to folder",
            window,
            cx,
            move |menu, _, cx| folder_items(menu, &view, id, &folders, cx),
        )
    };
    let Some(folder) = target.viewed_folder() else {
        return menu;
    };
    let folder_id = folder.id;
    menu.item(item(
        view,
        format!("Remove from {}", folder.name),
        "folder-minus",
        id,
        move |this, id, cx| this.set_in_folder(id, folder_id, false, cx),
    ))
}

/// One row per folder, tinted like the sidebar's. Checked ones hold the bookmark, and
/// choosing one again takes it out. The check sits on the right because a left check
/// would replace the folder's icon.
fn folder_items(
    menu: PopupMenu,
    view: &WeakEntity<MainView>,
    id: BookmarkId,
    folders: &[(Folder, bool)],
    cx: &App,
) -> PopupMenu {
    let theme = Theme::get(cx);
    let menu = menu
        .min_w(MENU_WIDTH)
        .max_w(MENU_WIDTH)
        .check_side(Side::Right);
    folders.iter().fold(menu, |menu, (folder, filed)| {
        let (folder_id, filed) = (folder.id, *filed);
        let view = view.clone();
        let tint = theme.folder_color(folder.color.0);
        menu.item(
            PopupMenuItem::new(shorten(&folder.name))
                .icon(menu_icon("folder").text_color(tint))
                .checked(filed)
                .on_click(move |_, _, cx| {
                    let _ = view.update(cx, |this, cx| {
                        this.set_in_folder(id, folder_id, !filed, cx);
                    });
                }),
        )
    })
}

fn shorten(name: &str) -> String {
    if name.chars().count() <= FOLDER_NAME_CHARS {
        return name.to_owned();
    }
    let kept: String = name.chars().take(FOLDER_NAME_CHARS - 1).collect();
    format!("{}…", kept.trim_end())
}

fn refreshing_items(menu: PopupMenu, view: &WeakEntity<MainView>, target: &Target) -> PopupMenu {
    let id = target.bookmark.id;
    let refreshing = target.refresh == Some(Refresh::Running);
    let thumbnail = if refreshing {
        "Refreshing thumbnail…"
    } else {
        "Refresh thumbnail"
    };
    let organizing = matches!(target.bookmark.status, Status::Queued | Status::Organizing);
    let organize = if organizing {
        "Organizing…"
    } else if target.bookmark.status.is_organized() {
        "Organize again"
    } else {
        "Organize"
    };
    menu.item(item(view, thumbnail, "image", id, MainView::refresh_thumbnail).disabled(refreshing))
        .item(item(view, organize, "sparkles", id, MainView::organize).disabled(organizing))
}

fn trash_items(menu: PopupMenu, view: &WeakEntity<MainView>, target: &Target) -> PopupMenu {
    let id = target.bookmark.id;
    opening_items(menu, view, target)
        .separator()
        .item(item(view, "Restore", "undo-2", id, |this, id, cx| {
            this.library.update(cx, |library, cx| {
                library.change(cx, move |store| store.restore(id));
            });
        }))
        .item(item(view, "Delete now", "trash", id, |this, id, cx| {
            this.library.update(cx, |library, cx| {
                library.change(cx, move |store| store.delete_for_good(id));
            });
        }))
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
        .icon(menu_icon(icon))
        .on_click(move |_, _, cx| {
            let _ = view.update(cx, |this, cx| act(this, id, cx));
        })
}

/// A Lucide icon from the bundled assets, the same set `components::icon` draws from.
fn menu_icon(name: &str) -> Icon {
    Icon::empty().path(format!("icons/{name}.svg"))
}

impl MainView {
    pub(super) fn set_in_folder(
        &mut self,
        id: BookmarkId,
        folder: FolderId,
        filed: bool,
        cx: &mut Context<'_, Self>,
    ) {
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| {
                if filed {
                    store.add_to_folder(id, folder)
                } else {
                    store.remove_from_folder(id, folder)
                }
            });
        });
    }

    /// Queues the bookmark for the AI, also when it was organized before.
    pub(super) fn organize(&mut self, id: BookmarkId, cx: &mut Context<'_, Self>) {
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| store.queue(id));
        });
        Organizer::wake(cx);
    }

    fn refresh_thumbnail(&mut self, id: BookmarkId, cx: &mut Context<'_, Self>) {
        let Some(url) = self.library.read(cx).bookmark(id).map(|b| b.url.clone()) else {
            return;
        };
        Thumbnails::entity(cx).update(cx, |thumbnails, cx| thumbnails.refresh(id, url, cx));
    }
}
