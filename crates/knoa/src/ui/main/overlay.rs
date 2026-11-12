//! Floating panels over the main screen: settings, view options, the workspace switcher,
//! the folder and workspace editors, and adding a link.
//! Most sit in the middle of the window; settings is a sheet nearly as big as the window,
//! and the workspace switcher drops down from its button. Clicking outside closes them.

use gpui::{
    AnyElement, AppContext as _, Context, Entity, FontWeight, InteractiveElement as _, IntoElement,
    MouseButton, ParentElement as _, Pixels, Size, Styled as _, Window, deferred, div,
    prelude::FluentBuilder as _, px, size,
};
use gpui_component::switch::Switch;
use knoa_core::{FolderId, WorkspaceId};

use super::{
    MainView, NewLink, OpenSettings,
    add_link::AddLink,
    folder_editor::FolderEditor,
    settings::{Page, SettingsView},
    sidebar::{TITLEBAR_HEIGHT, workspace_menu_left},
};
use crate::{
    state::settings::SettingsFile,
    ui::theme::{MARGIN, PANEL_RADIUS, Theme},
};

/// What floats over the main screen, if anything.
pub enum Overlay {
    Settings(Entity<SettingsView>),
    ViewOptions,
    Folder(Entity<FolderEditor>),
    /// The workspace switcher.
    Workspaces,
    AddLink(Entity<AddLink>),
}

impl MainView {
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
        self.show_settings(Page::General, window, cx);
    }
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
        let anchored = matches!(overlay, Overlay::Workspaces);
        let sheet = matches!(overlay, Overlay::Settings(_)).then(|| sheet_size(window));
        let panel: AnyElement = match overlay {
            Overlay::Settings(panel) => panel.clone().into_any_element(),
            Overlay::Folder(editor) => editor.clone().into_any_element(),
            Overlay::AddLink(panel) => panel.clone().into_any_element(),
            Overlay::ViewOptions => self.view_options(cx).into_any_element(),
            Overlay::Workspaces => self.workspace_switcher(cx).into_any_element(),
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
        let scrim = if anchored {
            scrim.child(
                panel
                    .absolute()
                    .top(below_titlebar())
                    .left(workspace_menu_left()),
            )
        } else if sheet.is_some() {
            scrim.child(panel.absolute().top(below_titlebar()).left(SHEET_INSET))
        } else {
            scrim.flex().items_center().justify_center().child(panel)
        };
        Some(deferred(scrim).with_priority(1).into_any_element())
    }

    fn view_options(&self, cx: &mut Context<'_, MainView>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let settings = SettingsFile::get(cx).clone();
        let row = |label: &'static str, id: &'static str, on: bool| {
            div()
                .flex()
                .items_center()
                .gap_3()
                .text_sm()
                .child(div().flex_1().child(label))
                .child(Switch::new(id).checked(on))
        };
        div()
            .w(px(300.))
            .p_5()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.faint)
                    .child("View"),
            )
            .child(
                row(
                    "Show summaries on cards",
                    "show-summaries",
                    settings.show_summaries,
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        let on = !SettingsFile::get(cx).show_summaries;
                        this.set_show_summaries(on, cx);
                    }),
                ),
            )
            .child(
                row(
                    "Show the Up next strip",
                    "show-up-next",
                    settings.show_up_next,
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|_, _, _, cx| {
                        SettingsFile::update(cx, |s| s.show_up_next = !s.show_up_next);
                        cx.notify();
                    }),
                ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted)
                    .child("Cards are grouped by date. Searching ranks by best match."),
            )
    }
}
