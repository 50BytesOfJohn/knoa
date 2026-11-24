//! The main screen as the window draws it: the main view with the sidebar floating over
//! its left edge, and the keys that work whatever has focus.
//!
//! Each view is drawn from its own cache and redrawn only when it is notified, so a hover
//! or an animation in one never rebuilds the other.

use gpui::{
    App, AppContext as _, Context, Entity, Focusable as _, InteractiveElement as _, IntoElement,
    KeyDownEvent, ParentElement as _, StyleRefinement, Styled as _, Window, div,
};
use knoa_core::{FolderId, Place};

use super::{
    FocusSearch, MainView, NewFolder, NewLink, SelectFolder, ShowDetails, SidebarEvent,
    SidebarView, overlay::Overlay,
};
use crate::{
    state::library::Library,
    ui::theme::{MARGIN, SIDEBAR_WIDTH},
};

/// The two views of the main screen. The main view owns the sidebar; this keeps both
/// handles so the window can draw them as siblings.
#[derive(Clone)]
pub struct MainScreen {
    main: Entity<MainView>,
    sidebar: Entity<SidebarView>,
}

impl MainScreen {
    pub fn new(library: Entity<Library>, window: &mut Window, cx: &mut App) -> MainScreen {
        let main = cx.new(|cx| MainView::new(library, window, cx));
        let sidebar = main.read(cx).sidebar.clone();
        MainScreen { main, sidebar }
    }

    /// Opens settings on the Account page, to sign in.
    pub fn open_account(&self, window: &mut Window, cx: &mut App) {
        self.main
            .update(cx, |main, cx| main.open_account(window, cx));
    }

    /// The keys live here rather than in the main view so they still work while
    /// something in the sidebar, such as a folder's menu, has focus.
    pub fn render(&self, window: &mut Window) -> impl IntoElement {
        let main = &self.main;
        div()
            .key_context("Main")
            .size_full()
            .relative()
            .on_key_down(window.listener_for(main, MainView::type_to_search))
            .on_action(window.listener_for(main, MainView::open_settings))
            .on_action(window.listener_for(main, MainView::focus_search))
            .on_action(window.listener_for(main, MainView::new_link))
            .on_action(window.listener_for(main, MainView::new_top_folder))
            .on_action(window.listener_for(main, MainView::show_details))
            .on_action(window.listener_for(main, MainView::select_folder))
            .child(main.clone().cached(StyleRefinement::default().size_full()))
            // After the main view, so the sidebar draws over the cards. The main view's
            // overlays are deferred, so they still draw over the sidebar.
            .child(self.sidebar.clone().cached(sidebar_frame()))
    }
}

fn sidebar_frame() -> StyleRefinement {
    StyleRefinement::default()
        .absolute()
        .top(MARGIN)
        .left(MARGIN)
        .bottom(MARGIN)
        .w(SIDEBAR_WIDTH)
}

impl MainView {
    pub(super) fn on_sidebar_event(
        &mut self,
        _: &Entity<SidebarView>,
        event: &SidebarEvent,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        match *event {
            SidebarEvent::Show(place) => self.set_place(place, window, cx),
            SidebarEvent::NewLink => self.new_link(&NewLink, window, cx),
            SidebarEvent::OpenWorkspaces => self.open_overlay(Overlay::Workspaces, cx),
            SidebarEvent::NewFolder(parent) => self.new_folder(parent, window, cx),
            SidebarEvent::EditFolder(id) => self.edit_folder(Some(id), window, cx),
            SidebarEvent::DeleteFolder(id) => self.confirm_delete_folder(id, window, cx),
            SidebarEvent::OpenSettings(page) => self.show_settings(page, window, cx),
        }
    }

    /// ⇧⌘N: a new folder at the top level.
    fn new_top_folder(
        &mut self,
        _: &NewFolder,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        self.close_viewer_first(window, cx);
        self.new_folder(None, window, cx);
    }

    /// Shows the folder's children in the sidebar.
    pub(super) fn expand_folder(&self, id: FolderId, cx: &mut Context<'_, MainView>) {
        self.sidebar
            .update(cx, |sidebar, cx| sidebar.expand(id, cx));
    }

    fn focus_search(
        &mut self,
        _: &FocusSearch,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        self.close_viewer_first(window, cx);
        self.search.update(cx, |input, cx| input.focus(window, cx));
    }

    /// Cmd-I: the detail panel for the selection, from the grid or the viewer.
    pub(super) fn show_details(
        &mut self,
        _: &ShowDetails,
        window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        self.close_viewer_first(window, cx);
        if let Some(id) = self.selected {
            self.select(id, cx);
        }
    }

    /// Typing anywhere searches: a printable key sends focus to the search field.
    fn type_to_search(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        let keys = &event.keystroke;
        if keys.modifiers.platform || keys.modifiers.control || self.overlay.is_some() {
            return;
        }
        let Some(text) = keys
            .key_char
            .as_ref()
            .filter(|c| c.chars().all(|ch| !ch.is_control()))
        else {
            return;
        };
        let typing_tag = self
            .tag_edits
            .as_ref()
            .is_some_and(|edits| edits.is_typing(window, cx));
        let typing_heading = self
            .heading_edit
            .as_ref()
            .is_some_and(|edit| edit.is_typing(window, cx));
        if typing_tag || typing_heading || self.search.read(cx).focus_handle(cx).is_focused(window)
        {
            return;
        }
        // `/` only focuses the search, as on the web; it isn't typed.
        let text = text.clone();
        self.search.update(cx, |input, cx| {
            input.focus(window, cx);
            if text != "/" {
                input.insert(text, window, cx);
            }
        });
        cx.stop_propagation();
    }

    /// ⌘1 to ⌘9: the folder the sidebar shows that key beside.
    fn select_folder(
        &mut self,
        action: &SelectFolder,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        if self.overlay.is_some() {
            return;
        }
        let n = action.0;
        if let Some(id) = self
            .sidebar
            .update(cx, |sidebar, cx| sidebar.nth_folder(n, cx))
        {
            self.set_place(Place::Folder(id), window, cx);
        }
    }
}
