//! The floating sidebar: the titlebar with the workspace and add-link buttons, places,
//! folders and the footer with the wordmark, the AI indicator and settings.
//!
//! It is its own view so that its hovers, drags and animations repaint only itself. It
//! asks the main view for anything outside it with a [`SidebarEvent`].

use gpui::{
    Context, Entity, EventEmitter, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, Pixels, Point, Render, StatefulInteractiveElement as _, Styled as _,
    Subscription, Window, div, px,
};
use knoa_core::{FolderId, Place};

mod drag_preview;
mod drop;
mod folder_menu;
mod folders;
mod footer;
mod outline;
mod places;
mod titlebar;

pub(super) use drag_preview::DragPreview;
pub(super) use drop::DraggedBookmark;

use super::settings::Page;
use crate::{
    services::{cloud::Cloud, organizer::Organizer},
    state::library::Library,
    ui::theme::{MARGIN, PANEL_RADIUS, SIDEBAR_WIDTH, Theme},
};
use drop::SidebarDrop;
use outline::Outline;

/// The sidebar's top row, shared with the traffic lights.
pub(super) const TITLEBAR_HEIGHT: Pixels = px(44.);
const TITLEBAR_GAP: Pixels = px(4.);
const BUTTON_PT: f32 = 26.;
const TITLEBAR_BUTTON: Pixels = px(BUTTON_PT);
/// The glyph inside a titlebar or footer button.
const BUTTON_ICON_PT: f32 = 16.;
const BUTTON_ICON: Pixels = px(BUTTON_ICON_PT);
const WORKSPACE_BUTTON_WIDTH: Pixels = px(40.);
const WORKSPACE_MARK: Pixels = px(20.);
const BORDER: Pixels = px(1.);
/// From the panel's inner edge to a row's highlight.
const ROW_INSET_PT: f32 = 8.;
const ROW_INSET: Pixels = px(ROW_INSET_PT);
const ROW_PADDING_PT: f32 = 8.;
const ROW_PADDING: Pixels = px(ROW_PADDING_PT);
/// From the panel's inner edge to the content: traffic lights, icons and labels on the
/// left, buttons and counts on the right. One column keeps the sidebar's edges straight.
const CONTENT_INSET_PT: f32 = ROW_INSET_PT + ROW_PADDING_PT;
const CONTENT_INSET: Pixels = px(CONTENT_INSET_PT);
/// The padding that puts a titlebar or footer button's glyph on the content column.
const BUTTON_PADDING: Pixels = px(CONTENT_INSET_PT - (BUTTON_PT - BUTTON_ICON_PT) / 2.);
/// The traffic lights' diameter, which `AppKit` sets.
const TRAFFIC_LIGHT: Pixels = px(14.);

/// Where `AppKit` should put the close button: centred in the titlebar row and on the
/// content column, so the lights sit as far in from the panel's edge as its icons.
pub fn traffic_light_position() -> Point<Pixels> {
    let left = MARGIN + BORDER + CONTENT_INSET;
    let top = MARGIN + BORDER + (TITLEBAR_HEIGHT - TRAFFIC_LIGHT) / 2.;
    gpui::point(left, top)
}

/// The workspace switcher's left edge in the window. It sits under the workspace button,
/// shifted left by the menu's and its rows' padding so the marks in the list line up under
/// the button's.
pub(super) fn workspace_menu_left() -> Pixels {
    let button_left = MARGIN + SIDEBAR_WIDTH
        - BORDER
        - BUTTON_PADDING
        - TITLEBAR_BUTTON
        - TITLEBAR_GAP
        - WORKSPACE_BUTTON_WIDTH;
    button_left - px(16.)
}

/// What the sidebar asks of the main view, which owns the place shown and the overlays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SidebarEvent {
    Show(Place),
    NewLink,
    OpenWorkspaces,
    NewFolder(Option<FolderId>),
    EditFolder(FolderId),
    DeleteFolder(FolderId),
    OpenSettings(Page),
}

pub(super) struct SidebarView {
    library: Entity<Library>,
    organizer: Entity<Organizer>,
    /// Which parents are collapsed, and the folder rows that leaves.
    outline: Outline,
    drop: SidebarDrop,
    /// The workspace switcher is open, so its button stays lit.
    workspaces_open: bool,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<SidebarEvent> for SidebarView {}

impl SidebarView {
    pub(super) fn new(library: Entity<Library>, cx: &mut Context<'_, SidebarView>) -> SidebarView {
        let organizer = Organizer::entity(cx);
        let subscriptions = vec![
            cx.observe(&library, |_, _, cx| cx.notify()),
            cx.observe(&organizer, |_, _, cx| cx.notify()),
            cx.observe(&Cloud::entity(cx), |_, _, cx| cx.notify()),
        ];
        SidebarView {
            library,
            organizer,
            outline: Outline::load(cx),
            drop: SidebarDrop::default(),
            workspaces_open: false,
            _subscriptions: subscriptions,
        }
    }

    pub(super) fn set_workspaces_open(&mut self, open: bool, cx: &mut Context<'_, SidebarView>) {
        if self.workspaces_open != open {
            self.workspaces_open = open;
            cx.notify();
        }
    }

    /// The `n`th visible folder, counting from 0, as ⌘1 to ⌘9 pick them.
    pub(super) fn nth_folder(
        &mut self,
        n: usize,
        cx: &mut Context<'_, SidebarView>,
    ) -> Option<FolderId> {
        let library = self.library.read(cx);
        self.outline
            .nth(library.folder_tree(), library.folders_generation(), n)
    }

    /// Opens the folder's parents, so its row is visible.
    pub(super) fn reveal(&mut self, id: FolderId, cx: &mut Context<'_, SidebarView>) {
        let path = self.library.read(cx).folder_tree().path(id);
        let parents: Vec<FolderId> = path.iter().rev().skip(1).map(|f| f.id).collect();
        if self.outline.reveal(&parents, cx) {
            cx.notify();
        }
    }

    /// Shows the folder's children.
    pub(super) fn expand(&mut self, id: FolderId, cx: &mut Context<'_, SidebarView>) {
        if self.outline.expand(id, cx) {
            cx.notify();
        }
    }
}

impl Render for SidebarView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let library = self.library.read(cx);
        let counts = library.counts();
        let current = library.place();
        let workspace = library
            .workspace()
            .map(|w| (w.name.clone(), w.color))
            .unwrap_or_default();

        div()
            .size_full()
            .flex()
            .flex_col()
            .rounded(PANEL_RADIUS)
            .bg(theme.panel)
            .border_1()
            .border_color(theme.hairline)
            .shadow_sm()
            .text_color(theme.text)
            .child(self.titlebar(workspace, cx))
            .child(div().mt_2().child(self.places(counts, current, cx)))
            .child(
                div()
                    .id("folders")
                    .mt_6()
                    .flex_1()
                    .overflow_y_scroll()
                    .track_scroll(&self.drop.scroll)
                    .on_scroll(cx.listener(|this, _, _, cx| {
                        if cx.has_active_drag() {
                            this.drop.end();
                            cx.notify();
                        }
                    }))
                    .child(
                        div()
                            .px(CONTENT_INSET)
                            .pb_1p5()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.faint)
                            .child("Folders"),
                    )
                    .child(self.folder_list(current, cx)),
            )
            .child(self.footer(counts.organizing, cx))
    }
}
