//! The main screen: sidebar, search pill, library cards and the detail panel.

mod add_link;
mod bookmark_menu;
mod cards;
mod content;
mod detail;
mod edits;
mod folder_editor;
mod groups;
mod masonry;
mod navigation;
mod opening;
mod overlay;
mod search;
mod settings;
mod sidebar;
mod trash;
mod workspaces;

use std::time::{Duration, Instant};

use gpui::{
    App, AppContext as _, ClipboardItem, Context, Entity, FocusHandle, Focusable as _,
    InteractiveElement as _, IntoElement, KeyBinding, KeyDownEvent, ParentElement as _, Render,
    ScrollHandle, Styled as _, Subscription, Task, Window, actions, div,
    prelude::FluentBuilder as _,
};
use gpui_component::input::{InputEvent, InputState};
use knoa_core::{BookmarkId, FolderId, Place, WorkspaceId};

use crate::{
    services::{organizer::Organizer, thumbnails::Thumbnails},
    state::{
        library::{Library, LibraryChanged},
        settings::SettingsFile,
    },
    ui::theme::Theme,
};
use add_link::AddLink;
use content::LayoutCache;
use detail::TagEdits;
use folder_editor::FolderEditor;
use settings::{Page, SettingsView};
pub use sidebar::traffic_light_position;

actions!(
    cards,
    [
        MoveLeft,
        MoveRight,
        MoveUp,
        MoveDown,
        OpenSelected,
        OpenSelectedInBackground,
        ToggleDetail,
        ToggleUpNext,
        CopyLink,
        DeleteSelected,
        Dismiss
    ]
);
actions!(knoa, [FocusSearch, OpenSettings, NewLink]);

/// Bindings for the card grid. They live in their own key context so they never fire while
/// typing in the search field.
pub fn bind_keys(cx: &mut App) {
    let cards = Some("Cards");
    cx.bind_keys([
        KeyBinding::new("left", MoveLeft, cards),
        KeyBinding::new("right", MoveRight, cards),
        KeyBinding::new("up", MoveUp, cards),
        KeyBinding::new("down", MoveDown, cards),
        KeyBinding::new("enter", OpenSelected, cards),
        KeyBinding::new("cmd-o", OpenSelected, cards),
        KeyBinding::new("cmd-down", OpenSelected, cards),
        KeyBinding::new("cmd-enter", OpenSelectedInBackground, cards),
        KeyBinding::new("space", ToggleDetail, cards),
        KeyBinding::new("l", ToggleUpNext, cards),
        KeyBinding::new("cmd-c", CopyLink, cards),
        KeyBinding::new("backspace", DeleteSelected, cards),
        KeyBinding::new("cmd-backspace", DeleteSelected, cards),
        KeyBinding::new("escape", Dismiss, cards),
        KeyBinding::new("escape", Dismiss, Some("Settings")),
        KeyBinding::new("cmd-f", FocusSearch, None),
        KeyBinding::new("cmd-k", FocusSearch, None),
        KeyBinding::new("cmd-,", OpenSettings, None),
        KeyBinding::new("cmd-n", NewLink, None),
    ]);
}

/// What floats over the main screen, if anything.
pub enum Overlay {
    Settings(Entity<SettingsView>),
    ViewOptions,
    Folder(Entity<FolderEditor>),
    /// The workspace switcher.
    Workspaces,
    AddLink(Entity<AddLink>),
}

pub struct MainView {
    library: Entity<Library>,
    organizer: Entity<Organizer>,
    search: Entity<InputState>,
    cards_focus: FocusHandle,
    scroll: ScrollHandle,
    selected: Option<BookmarkId>,
    detail_open: bool,
    /// The bookmark whose folder chips show every folder, to add it to another.
    folder_picker: Option<BookmarkId>,
    tag_edits: Option<TagEdits>,
    /// The bookmark whose link was just copied, until the task clears the confirmation.
    copied: Option<(BookmarkId, Task<()>)>,
    overlay: Option<Overlay>,
    /// Empty Trash was clicked once; the next click deletes for good.
    confirm_empty_trash: bool,
    layout: LayoutCache,
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
            cx.new(|cx| InputState::new(window, cx).placeholder("Just start typing to search"));
        let organizer = Organizer::entity(cx);
        let subscriptions = vec![
            cx.subscribe(&search, |this, input, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    let text = input.read(cx).value().to_string();
                    this.library
                        .update(cx, |library, cx| library.set_search(&text, cx));
                    this.scroll
                        .set_offset(gpui::point(gpui::px(0.), gpui::px(0.)));
                }
            }),
            cx.subscribe(&library, |this, _, _: &LibraryChanged, cx| {
                this.on_library_changed(cx);
            }),
            cx.observe(&organizer, |_, _, cx| cx.notify()),
            cx.observe(&Thumbnails::entity(cx), |_, _, cx| cx.notify()),
            cx.observe_window_activation(window, Self::keep_focus),
        ];
        let cards_focus = cx.focus_handle();
        cards_focus.focus(window, cx);
        MainView {
            library,
            organizer,
            search,
            cards_focus,
            scroll: ScrollHandle::new(),
            selected: None,
            detail_open: false,
            folder_picker: None,
            tag_edits: None,
            copied: None,
            overlay: None,
            confirm_empty_trash: false,
            layout: LayoutCache::default(),
            keep_focus_until: None,
            _subscriptions: subscriptions,
        }
    }

    /// The selection outlives filing it elsewhere, so it can be filed again or opened. It
    /// ends when the bookmark is deleted, or trashed or restored: those send it away.
    fn on_library_changed(&mut self, cx: &mut Context<'_, MainView>) {
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

    pub fn set_place(&mut self, place: Place, window: &mut Window, cx: &mut Context<'_, MainView>) {
        self.confirm_empty_trash = false;
        self.library
            .update(cx, |library, cx| library.set_place(place, cx));
        self.scroll
            .set_offset(gpui::point(gpui::px(0.), gpui::px(0.)));
        self.cards_focus.focus(window, cx);
    }

    pub fn open_overlay(&mut self, overlay: Overlay, cx: &mut Context<'_, MainView>) {
        self.overlay = Some(overlay);
        cx.notify();
    }

    pub fn close_overlay(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        self.overlay = None;
        self.cards_focus.focus(window, cx);
        cx.notify();
    }

    pub fn edit_folder(
        &mut self,
        folder: Option<FolderId>,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        let existing = folder.and_then(|id| {
            self.library
                .read(cx)
                .folders()
                .iter()
                .find(|f| f.id == id)
                .cloned()
        });
        let library = self.library.clone();
        let this = cx.entity().downgrade();
        let editor = cx.new(|cx| FolderEditor::new(existing, library, this, window, cx));
        self.open_overlay(Overlay::Folder(editor), cx);
    }

    /// Opens settings on a workspace's page; `None` on the page that creates one.
    pub fn edit_workspace(
        &mut self,
        workspace: Option<WorkspaceId>,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        let page = workspace.map_or(Page::NewWorkspace, Page::Workspace);
        self.show_settings(page, window, cx);
    }

    fn show_settings(&mut self, page: Page, window: &mut Window, cx: &mut Context<'_, MainView>) {
        if let Some(Overlay::Settings(settings)) = &self.overlay {
            settings.update(cx, |settings, cx| settings.show(page, window, cx));
            return;
        }
        let library = self.library.clone();
        let this = cx.entity().downgrade();
        let settings = cx.new(|cx| SettingsView::new(page, library, this, window, cx));
        self.open_overlay(Overlay::Settings(settings), cx);
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

    fn new_link(&mut self, _: &NewLink, window: &mut Window, cx: &mut Context<'_, MainView>) {
        let library = self.library.clone();
        let this = cx.entity().downgrade();
        let panel = cx.new(|cx| AddLink::new(library, this, window, cx));
        self.open_overlay(Overlay::AddLink(panel), cx);
    }

    fn open_settings(
        &mut self,
        _: &OpenSettings,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        self.show_settings(Page::General, window, cx);
    }

    fn focus_search(
        &mut self,
        _: &FocusSearch,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        self.search.update(cx, |input, cx| input.focus(window, cx));
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
        if typing_tag || self.search.read(cx).focus_handle(cx).is_focused(window) {
            return;
        }
        let text = text.clone();
        self.search.update(cx, |input, cx| {
            input.focus(window, cx);
            input.insert(text, window, cx);
        });
        cx.stop_propagation();
    }

    fn dismiss(&mut self, _: &Dismiss, window: &mut Window, cx: &mut Context<'_, MainView>) {
        if self.overlay.is_some() {
            self.close_overlay(window, cx);
        } else if !self.search.read(cx).value().is_empty() {
            self.search
                .update(cx, |input, cx| input.set_value("", window, cx));
            self.library
                .update(cx, |library, cx| library.set_search("", cx));
        } else {
            self.detail_open = false;
            cx.notify();
        }
    }

    fn toggle_detail(&mut self, _: &ToggleDetail, _: &mut Window, cx: &mut Context<'_, MainView>) {
        self.detail_open = !self.detail_open && self.selected.is_some();
        cx.notify();
    }

    fn copy_link(&mut self, _: &CopyLink, _: &mut Window, cx: &mut Context<'_, MainView>) {
        if let Some(id) = self.selected {
            self.copy_link_of(id, cx);
        }
    }

    fn copy_link_of(&mut self, id: BookmarkId, cx: &mut Context<'_, MainView>) {
        if let Some(url) = self.url_of(id, cx) {
            self.copy_to_clipboard(id, url, cx);
        }
    }

    fn url_of(&self, id: BookmarkId, cx: &App) -> Option<String> {
        self.library.read(cx).bookmark(id).map(|b| b.url.clone())
    }

    /// Copies a bookmark's link and confirms it on the detail panel's copy button for a
    /// moment.
    fn copy_to_clipboard(&mut self, id: BookmarkId, url: String, cx: &mut Context<'_, MainView>) {
        cx.write_to_clipboard(ClipboardItem::new_string(url));
        let reset = cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(1500))
                .await;
            let _ = this.update(cx, |this, cx| {
                this.copied = None;
                cx.notify();
            });
        });
        self.copied = Some((id, reset));
        cx.notify();
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
            .key_context("Main")
            .size_full()
            .relative()
            .bg(theme.canvas)
            .text_color(theme.text)
            .on_key_down(cx.listener(Self::type_to_search))
            .on_action(cx.listener(Self::open_settings))
            .on_action(cx.listener(Self::focus_search))
            .on_action(cx.listener(Self::new_link))
            .child(self.render_content(show_detail, window, cx))
            .child(self.render_search(show_detail, window, cx))
            .child(self.render_sidebar(window, cx))
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
