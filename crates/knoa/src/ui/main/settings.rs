//! Settings: one sheet over the main window, with its own sidebar of pages. App-wide pages
//! (General, Appearance, AI) come first, then one page per workspace.

mod ai;
mod appearance;
mod form;
mod general;
mod nav;
mod workspace;

use gpui::{
    AppContext as _, Context, Entity, FocusHandle, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, StatefulInteractiveElement as _, Styled as _, Subscription,
    WeakEntity, Window, div, px,
};
use knoa_core::WorkspaceId;

use super::{Dismiss, MainView};
use crate::{
    state::library::{Library, LibraryChanged},
    ui::{components::icon, theme::Theme},
};
use ai::AiPage;
use workspace::{WorkspacePage, WorkspacePageEvent};

/// The widest a page's column gets; past this, rows are hard to read across.
const PAGE_WIDTH: gpui::Pixels = px(720.);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    General,
    Appearance,
    Ai,
    Workspace(WorkspaceId),
    NewWorkspace,
}

pub struct SettingsView {
    page: Page,
    library: Entity<Library>,
    main: WeakEntity<MainView>,
    focus: FocusHandle,
    ai: Entity<AiPage>,
    /// The page of the workspace shown, while one is.
    workspace: Option<Entity<WorkspacePage>>,
    _subscriptions: Vec<Subscription>,
}

impl SettingsView {
    pub fn new(
        page: Page,
        library: Entity<Library>,
        main: WeakEntity<MainView>,
        window: &mut Window,
        cx: &mut Context<'_, SettingsView>,
    ) -> SettingsView {
        let ai = cx.new(|cx| AiPage::new(library.clone(), window, cx));
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        // However the sheet closes (Esc, a click outside, the close button), edits not yet
        // saved are saved.
        cx.on_release(|this: &mut SettingsView, cx| this.commit(cx))
            .detach();
        let subscriptions = vec![cx.subscribe_in(
            &library,
            window,
            |this, _, _: &LibraryChanged, window, cx| this.on_library_changed(window, cx),
        )];
        let mut view = SettingsView {
            page: Page::General,
            library,
            main,
            focus,
            ai,
            workspace: None,
            _subscriptions: subscriptions,
        };
        view.show(page, window, cx);
        view
    }

    /// Switches to `page`, saving the workspace page being left.
    pub fn show(&mut self, page: Page, window: &mut Window, cx: &mut Context<'_, SettingsView>) {
        self.commit(cx);
        self.workspace = match page {
            Page::Workspace(id) => {
                let existing = self.find_workspace(id, cx);
                Some(self.workspace_page(existing, window, cx))
            }
            Page::NewWorkspace => Some(self.workspace_page(None, window, cx)),
            _ => None,
        };
        self.page = page;
        cx.notify();
    }

    /// Saves edits that aren't saved yet.
    fn commit(&self, cx: &mut gpui::App) {
        if let Some(page) = &self.workspace {
            page.update(cx, WorkspacePage::save);
        }
    }

    fn close(&mut self, window: &mut Window, cx: &mut Context<'_, SettingsView>) {
        let _ = self
            .main
            .update(cx, |main, cx| main.close_overlay(window, cx));
    }

    fn find_workspace(
        &self,
        id: WorkspaceId,
        cx: &Context<'_, SettingsView>,
    ) -> Option<knoa_core::Workspace> {
        self.library
            .read(cx)
            .workspaces()
            .iter()
            .find(|w| w.id == id)
            .cloned()
    }

    fn workspace_page(
        &self,
        existing: Option<knoa_core::Workspace>,
        window: &mut Window,
        cx: &mut Context<'_, SettingsView>,
    ) -> Entity<WorkspacePage> {
        let library = self.library.clone();
        let page = cx.new(|cx| WorkspacePage::new(existing, library, window, cx));
        // The page stays: it turns into the new workspace's page by itself.
        cx.subscribe(&page, |this, _, event, cx| match event {
            WorkspacePageEvent::Created(id) => {
                this.page = Page::Workspace(*id);
                cx.notify();
            }
        })
        .detach();
        page
    }

    /// A workspace deleted here, or anywhere else, takes its page with it.
    fn on_library_changed(&mut self, window: &mut Window, cx: &mut Context<'_, SettingsView>) {
        if let Page::Workspace(id) = self.page
            && self.find_workspace(id, cx).is_none()
        {
            self.workspace = None;
            self.show(Page::General, window, cx);
        }
        cx.notify();
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        div()
            .id("settings")
            .key_context("Settings")
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &Dismiss, window, cx| this.close(window, cx)))
            .size_full()
            .flex()
            .child(self.nav(cx))
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(
                        div()
                            .id("settings-page")
                            .size_full()
                            .overflow_y_scroll()
                            .child(
                                div()
                                    .mx_auto()
                                    .max_w(PAGE_WIDTH)
                                    .px_10()
                                    .pt_10()
                                    .pb_12()
                                    .child(self.page_body(cx)),
                            ),
                    )
                    .child(self.close_button(&theme, cx)),
            )
    }
}

impl SettingsView {
    fn page_body(&self, cx: &mut Context<'_, SettingsView>) -> gpui::AnyElement {
        match (self.page, &self.workspace) {
            (Page::General, _) => general::page(cx).into_any_element(),
            (Page::Appearance, _) => appearance::page(cx).into_any_element(),
            (Page::Ai, _) => self.ai.clone().into_any_element(),
            (Page::Workspace(_) | Page::NewWorkspace, Some(page)) => {
                page.clone().into_any_element()
            }
            (Page::Workspace(_) | Page::NewWorkspace, None) => div().into_any_element(),
        }
    }

    fn close_button(&self, theme: &Theme, cx: &mut Context<'_, SettingsView>) -> impl IntoElement {
        div()
            .id("close-settings")
            .absolute()
            .top_4()
            .right_4()
            .size(px(28.))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.bg(theme.hover))
            .child(icon("x", px(16.), theme.muted))
            .on_click(cx.listener(|this, _, window, cx| this.close(window, cx)))
    }
}
