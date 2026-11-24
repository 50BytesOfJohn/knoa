//! The main screen: sidebar, search pill, library cards and the detail panel.

mod add_link;
mod bookmark_menu;
mod cards;
mod clipboard;
mod content;
mod delete_folder;
mod detail;
mod display;
mod edits;
mod events;
mod field_display;
mod folder_editor;
mod groups;
mod keys;
mod masonry;
pub(super) mod menu_trigger;
mod navigation;
mod opening;
mod overlay;
mod picture;
mod screen;
mod search;
mod settings;
mod sidebar;
mod tiles;
mod trash;
mod viewer;
mod workspaces;

use std::time::Instant;

use gpui::{
    App, AppContext as _, Context, Entity, FocusHandle, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, ScrollHandle, Styled as _, Subscription, Task, Window, div,
    prelude::FluentBuilder as _,
};
use gpui_component::input::InputState;
use knoa_core::{BookmarkId, Place, WorkspaceId};

use crate::{
    i18n::{self, t},
    services::thumbnails::Thumbnails,
    state::{
        library::{Library, LibraryChanged},
        settings::SettingsFile,
    },
    ui::theme::Theme,
};
use content::{DecodedImages, LayoutCache};
use detail::{HeadingEdit, TagEdits};
use events::EventsPage;
use keys::{
    CloseViewer, CopyImage, CopyLink, DeleteSelected, Dismiss, FocusSearch, MoveDown, MoveLeft,
    MoveRight, MoveUp, NewFolder, NewLink, NextImage, OpenSelected, OpenSelectedInBackground,
    PreviousImage, QuickLook, SaveImage, SelectFolder, ShowDetails, ToggleUpNext,
};
pub use keys::{OpenSettings, bind_keys};
use overlay::Overlay;
pub use screen::MainScreen;
use search::PillState;
pub use sidebar::traffic_light_position;
use sidebar::{SidebarEvent, SidebarView};

pub struct MainView {
    library: Entity<Library>,
    sidebar: Entity<SidebarView>,
    search: Entity<InputState>,
    pill: PillState,
    cards_focus: FocusHandle,
    /// Held by the image viewer while it is open, so its keys replace the grid's.
    viewer_focus: FocusHandle,
    scroll: ScrollHandle,
    selected: Option<BookmarkId>,
    detail_open: bool,
    /// The bookmark whose folder chips show every folder, to add it to another.
    folder_picker: Option<BookmarkId>,
    tag_edits: Option<TagEdits>,
    /// The title and description fields, while they're open. Left on a bookmark no longer
    /// selected, they save once the blur that took focus away is handled.
    heading_edit: Option<HeadingEdit>,
    /// The bookmark whose link was just copied, until the task clears the confirmation.
    copied: Option<(BookmarkId, Task<()>)>,
    overlay: Option<Overlay>,
    /// Empty Trash was clicked once; the next click deletes for good.
    confirm_empty_trash: bool,
    layout: LayoutCache,
    events: EventsPage,
    decoded: DecodedImages,
    /// Until when Knoa takes focus back from a browser it opened a link in behind itself.
    keep_focus_until: Option<Instant>,
    _subscriptions: Vec<Subscription>,
}

impl MainView {
    pub fn new(
        library: Entity<Library>,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> MainView {
        let search =
            cx.new(|cx| InputState::new(window, cx).placeholder(t!("main-search-placeholder")));
        let sidebar = cx.new(|cx| SidebarView::new(library.clone(), cx));
        let subscriptions = vec![
            cx.subscribe_in(&search, window, Self::on_search_event),
            cx.subscribe_in(
                &library,
                window,
                |this, _, _: &LibraryChanged, window, cx| {
                    this.on_library_changed(window, cx);
                },
            ),
            cx.subscribe_in(&sidebar, window, Self::on_sidebar_event),
            cx.observe(&Thumbnails::entity(cx), |_, _, cx| cx.notify()),
            cx.observe_window_activation(window, Self::keep_focus),
            i18n::on_change(window, cx, Self::translate),
        ];
        let cards_focus = cx.focus_handle();
        cards_focus.focus(window, cx);
        MainView {
            library,
            sidebar,
            search,
            pill: PillState::default(),
            cards_focus,
            viewer_focus: cx.focus_handle(),
            scroll: ScrollHandle::new(),
            selected: None,
            detail_open: false,
            folder_picker: None,
            tag_edits: None,
            heading_edit: None,
            copied: None,
            overlay: None,
            confirm_empty_trash: false,
            layout: LayoutCache::default(),
            events: EventsPage::new(),
            decoded: DecodedImages::default(),
            keep_focus_until: None,
            _subscriptions: subscriptions,
        }
    }

    /// The selection outlives filing it elsewhere, so it can be filed again or opened. It
    /// ends when the bookmark is deleted, or trashed or restored: those send it away.
    fn on_library_changed(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        self.sync_input(window, cx);
        let library = self.library.read(cx);
        let in_trash = library.place() == Place::Trash;
        let still_here = |id| {
            library
                .bookmark(id)
                .is_some_and(|b| b.deleted_at.is_some() == in_trash)
        };
        if let Some(id) = self.selected
            && !still_here(id)
        {
            self.set_selected(None, cx);
            self.detail_open = false;
        }
        cx.notify();
    }

    pub fn select(&mut self, id: BookmarkId, cx: &mut Context<'_, MainView>) {
        self.set_selected(Some(id), cx);
        self.detail_open = true;
        cx.notify();
    }

    /// The only way to change the selection: the library has to know it to keep the
    /// bookmark loaded.
    fn set_selected(&mut self, id: Option<BookmarkId>, cx: &mut Context<'_, MainView>) {
        self.selected = id;
        self.library
            .update(cx, |library, _| library.set_selected(id));
    }

    /// Shows `place`. A folder chosen from anywhere but its own row, such as a menu or
    /// ⌘1 to ⌘9, has its parents opened so the selection is visible.
    pub fn set_place(&mut self, place: Place, window: &mut Window, cx: &mut Context<'_, MainView>) {
        self.show_place(place, cx);
        self.cards_focus.focus(window, cx);
    }

    /// [`MainView::set_place`] without moving focus, for the search pill.
    fn show_place(&mut self, place: Place, cx: &mut Context<'_, MainView>) {
        self.confirm_empty_trash = false;
        if let Place::Folder(id) = place {
            self.sidebar
                .update(cx, |sidebar, cx| sidebar.reveal(id, cx));
        }
        self.library
            .update(cx, |library, cx| library.set_place(place, cx));
        self.scroll
            .set_offset(gpui::point(gpui::px(0.), gpui::px(0.)));
    }

    /// Summaries change card heights, so the masonry layout is redone.
    pub fn set_show_summaries(&mut self, on: bool, cx: &mut Context<'_, MainView>) {
        SettingsFile::update(cx, |s| s.show_summaries = on);
        self.layout = LayoutCache::default();
        cx.notify();
    }

    pub fn switch_workspace(
        &mut self,
        id: WorkspaceId,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        self.library
            .update(cx, |library, cx| library.set_workspace(id, cx));
        self.set_selected(None, cx);
        self.detail_open = false;
        self.scroll
            .set_offset(gpui::point(gpui::px(0.), gpui::px(0.)));
        self.close_overlay(window, cx);
    }

    fn dismiss(&mut self, _: &Dismiss, window: &mut Window, cx: &mut Context<'_, MainView>) {
        if self.overlay.is_some() {
            self.close_overlay(window, cx);
        } else if !self.library.read(cx).search().is_empty() {
            self.clear_search(window, cx);
        } else {
            self.detail_open = false;
            cx.notify();
        }
    }

    fn toggle_detail(&mut self, cx: &mut Context<'_, MainView>) {
        self.detail_open = !self.detail_open && self.selected.is_some();
        cx.notify();
    }

    fn url_of(&self, id: BookmarkId, cx: &App) -> Option<String> {
        self.library.read(cx).bookmark(id).map(|b| b.url.clone())
    }

    fn toggle_up_next(&mut self, _: &ToggleUpNext, _: &mut Window, cx: &mut Context<'_, MainView>) {
        if let Some(id) = self.selected {
            self.set_up_next(id, cx);
        }
    }
}

impl Render for MainView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        self.forget_stale_tag_edits();
        let theme = Theme::get(cx).clone();
        let show_detail = self.detail_open && self.selected.is_some();
        let overlay = self.render_overlay(window, cx);
        div()
            .id("main")
            .size_full()
            .relative()
            .bg(theme.canvas)
            .text_color(theme.text)
            .child(self.render_content(show_detail, window, cx))
            .child(self.render_search(show_detail, window, cx))
            .child(self.render_capsule(show_detail, window, cx))
            .when_some(show_detail.then_some(()), |el, ()| {
                el.child(self.render_detail(window, cx))
            })
            .children(overlay)
    }
}

impl MainView {
    fn show_summaries(cx: &App) -> bool {
        SettingsFile::get(cx).show_summaries
    }
}
