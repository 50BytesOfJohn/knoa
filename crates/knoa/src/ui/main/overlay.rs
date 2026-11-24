//! Floating panels over the main screen: settings, Display, the workspace switcher,
//! the folder and workspace editors, and adding a link.
//! Most sit in the middle of the window; settings is a sheet nearly as big as the window,
//! and the workspace switcher, Display and the folded search tokens drop down from their
//! buttons. Clicking outside closes them.

use gpui::{
    Animation, AnimationExt as _, AnyElement, AppContext as _, Context, Entity,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement as _, Pixels, Size,
    Styled as _, Window, deferred, div, prelude::FluentBuilder as _, px, size,
};
use knoa_core::{Folder, FolderId, WorkspaceId};

use super::{
    MainView, NewLink, OpenSettings,
    add_link::AddLink,
    display,
    folder_editor::FolderEditor,
    search::PILL_HEIGHT,
    settings::{Page, SettingsView},
    sidebar::{TITLEBAR_HEIGHT, workspace_menu_left},
    viewer::Viewer,
};
use crate::ui::{
    motion,
    theme::{MARGIN, PANEL_RADIUS, Theme},
};

/// What floats over the main screen, if anything.
pub enum Overlay {
    Settings(Entity<SettingsView>),
    /// How the library looks, dropping down from the corner capsule.
    Display,
    Folder(Entity<FolderEditor>),
    /// The workspace switcher.
    Workspaces,
    AddLink(Entity<AddLink>),
    /// The full-window image viewer.
    Viewer(Viewer),
    /// Asking before a folder and its subfolders are deleted.
    DeleteFolder(FolderId),
    /// The search tokens folded into "+N", dropping down from it at `left`.
    FoldedTokens {
        left: Pixels,
    },
}

impl MainView {
    pub fn open_overlay(&mut self, overlay: Overlay, cx: &mut Context<'_, MainView>) {
        self.set_overlay(Some(overlay), cx);
    }

    pub fn close_overlay(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        self.set_overlay(None, cx);
        self.cards_focus.focus(window, cx);
    }

    /// The only way to change the overlay: the sidebar lights the workspace button while
    /// the switcher is open.
    pub(super) fn set_overlay(&mut self, overlay: Option<Overlay>, cx: &mut Context<'_, MainView>) {
        let switcher = matches!(overlay, Some(Overlay::Workspaces));
        self.overlay = overlay;
        self.sidebar
            .update(cx, |sidebar, cx| sidebar.set_workspaces_open(switcher, cx));
        cx.notify();
    }

    /// Opens the folder editor on an existing folder.
    pub fn edit_folder(
        &mut self,
        folder: Option<FolderId>,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        let tree = self.library.read(cx).folder_tree();
        let existing = folder.and_then(|id| {
            let shown_parent = tree.parent(id).map(|f| f.id);
            tree.get(id).map(|f| Folder {
                parent: shown_parent,
                ..f.clone()
            })
        });
        let parent = existing.as_ref().and_then(|f| f.parent);
        self.open_folder_editor(existing, parent, window, cx);
    }

    /// Opens the folder editor on a new folder under `parent`, or at the top level.
    pub fn new_folder(
        &mut self,
        parent: Option<FolderId>,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        self.open_folder_editor(None, parent, window, cx);
    }

    fn open_folder_editor(
        &mut self,
        existing: Option<Folder>,
        parent: Option<FolderId>,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        let library = self.library.clone();
        let this = cx.entity().downgrade();
        let editor = cx.new(|cx| FolderEditor::new(existing, parent, library, this, window, cx));
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

    /// Opens settings on the Account page, to sign in.
    pub fn open_account(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        self.show_settings(Page::Account, window, cx);
    }

    pub(super) fn show_settings(
        &mut self,
        page: Page,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        if let Some(Overlay::Settings(settings)) = &self.overlay {
            settings.update(cx, |settings, cx| settings.show(page, window, cx));
            return;
        }
        let library = self.library.clone();
        let this = cx.entity().downgrade();
        let settings = cx.new(|cx| SettingsView::new(page, library, this, window, cx));
        self.open_overlay(Overlay::Settings(settings), cx);
    }

    pub(super) fn new_link(
        &mut self,
        _: &NewLink,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        self.close_viewer_first(window, cx);
        let library = self.library.clone();
        let this = cx.entity().downgrade();
        let panel = cx.new(|cx| AddLink::new(library, this, window, cx));
        self.open_overlay(Overlay::AddLink(panel), cx);
    }

    pub(super) fn open_settings(
        &mut self,
        _: &OpenSettings,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        self.close_viewer_first(window, cx);
        self.show_settings(Page::General, window, cx);
    }
}

/// From the switcher's outer edge to its workspace marks: its border, padding and a row's
/// padding.
const SWITCHER_MARK_INSET: Pixels = px(21.);

/// A drop-down fading in and sliding down from its button to `top` as it opens.
fn drop_down(panel: gpui::Stateful<gpui::Div>, top: Pixels) -> impl IntoElement {
    panel.with_animation(
        "drop-down-open",
        Animation::new(motion::MENU_OPEN),
        move |panel, delta| {
            panel
                .opacity(delta)
                .top(top - px(motion::MENU_SLIDE * (1. - delta)))
        },
    )
}

/// Settings leaves a band of the main screen showing on every side, so it reads as a sheet
/// over the library rather than a new screen.
const SHEET_INSET: Pixels = px(36.);

/// Like a native sheet, settings hangs below the titlebar, so the traffic lights stay on the
/// dimmed window instead of floating over the sheet.
fn below_titlebar() -> Pixels {
    MARGIN + TITLEBAR_HEIGHT
}

fn sheet_size(window: &Window) -> Size<Pixels> {
    let viewport = window.viewport_size();
    size(
        viewport.width - SHEET_INSET * 2.,
        viewport.height - below_titlebar() - SHEET_INSET,
    )
}

impl MainView {
    pub(super) fn render_overlay(
        &mut self,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> Option<AnyElement> {
        let theme = Theme::get(cx).clone();
        let overlay = self.overlay.as_ref()?;
        let anchored = matches!(
            overlay,
            Overlay::Workspaces | Overlay::FoldedTokens { .. } | Overlay::Display
        );
        let sheet = matches!(overlay, Overlay::Settings(_)).then(|| sheet_size(window));
        let panel: AnyElement = match overlay {
            Overlay::Settings(panel) => panel.clone().into_any_element(),
            Overlay::Folder(editor) => editor.clone().into_any_element(),
            Overlay::AddLink(panel) => panel.clone().into_any_element(),
            Overlay::Display => self.display_popover(cx).into_any_element(),
            Overlay::Workspaces => self.workspace_switcher(cx).into_any_element(),
            Overlay::FoldedTokens { .. } => self.folded_tokens_popover(cx).into_any_element(),
            Overlay::DeleteFolder(id) => self.delete_folder_prompt(*id, cx).into_any_element(),
            // Not a panel: the viewer fills the window with its own backdrop.
            Overlay::Viewer(viewer) => {
                let viewer = self.render_viewer(viewer.bookmark, viewer.notice(), window, cx);
                return Some(deferred(viewer).with_priority(1).into_any_element());
            }
        };
        let panel = div()
            .id("overlay-panel")
            .occlude()
            .rounded(PANEL_RADIUS)
            .bg(theme.panel)
            .border_1()
            .border_color(theme.hairline)
            .shadow_lg()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .when_some(sheet, |s, sheet| s.w(sheet.width).h(sheet.height))
            .child(panel);
        // A drop-down keeps the window undimmed: it belongs to its button, not the window.
        let scrim_opacity = match (anchored, theme.dark) {
            (true, _) => 0.,
            (false, true) => 0.35,
            (false, false) => 0.12,
        };
        let scrim = div()
            .id("scrim")
            .absolute()
            .inset_0()
            .bg(gpui::black().opacity(scrim_opacity))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| this.close_overlay(window, cx)),
            );
        let scrim = if let Some(Overlay::Display) = self.overlay {
            let show_detail = self.detail_open && self.selected.is_some();
            let (top, right) = display::popover_anchor(show_detail, window);
            scrim.child(drop_down(
                panel.rounded(px(14.)).absolute().right(right),
                top,
            ))
        } else if let Some(Overlay::FoldedTokens { left }) = self.overlay {
            let top = MARGIN + px(PILL_HEIGHT) + px(6.);
            scrim.child(drop_down(panel.rounded(px(14.)).absolute().left(left), top))
        } else if anchored {
            let left = workspace_menu_left(SWITCHER_MARK_INSET);
            scrim.child(drop_down(panel.absolute().left(left), below_titlebar()))
        } else if sheet.is_some() {
            scrim.child(panel.absolute().top(below_titlebar()).left(SHEET_INSET))
        } else {
            scrim.flex().items_center().justify_center().child(panel)
        };
        Some(deferred(scrim).with_priority(1).into_any_element())
    }
}
