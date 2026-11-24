//! The floating sidebar: the titlebar with the workspace and add-link buttons, places,
//! folders, a status card when the AI or sync has something to say, and the footer with
//! the account menu and settings.
//!
//! It is its own view so that its hovers, drags and animations repaint only itself. It
//! asks the main view for anything outside it with a [`SidebarEvent`].

use gpui::{
    Context, Entity, EventEmitter, InteractiveElement as _, IntoElement, ParentElement as _,
    Pixels, Point, Render, StatefulInteractiveElement as _, Styled as _, Subscription, Window, div,
    px,
};
use knoa_core::{FolderId, Place};

mod account;
mod account_menu;
mod drag_preview;
mod drop;
mod folder_menu;
mod folder_row;
mod folders;
mod footer;
mod outline;
mod places;
mod row;
mod shown;
mod status;
mod status_card;
mod titlebar;

pub(super) use account_menu::bind_account_menu_keys;
pub(super) use drag_preview::DragPreview;
pub(super) use drop::DraggedBookmark;

use super::settings::Page;
use crate::{
    services::{
        cloud::Cloud,
        organizer::{AiStatus, Organizer},
    },
    state::{library::Library, settings::SettingsFile},
    ui::theme::{MARGIN, PANEL_RADIUS, Theme},
};
use account::Identity;
use account_menu::AccountMenu;
use drop::SidebarDrop;
use outline::Outline;
use places::PlaceRows;
pub(super) use places::place_icon;
use shown::{CloudShown, LibraryShown};
use status_card::StatusCard;

/// The sidebar's top row, shared with the traffic lights.
pub(super) const TITLEBAR_HEIGHT: Pixels = px(44.);
const BORDER: Pixels = px(1.);
/// From the panel's inner edge to a row's highlight, and to the titlebar's and footer's
/// controls.
const INSET: Pixels = px(8.);
/// The traffic lights' left edge from the panel's inner edge.
const LIGHTS_INSET: Pixels = px(16.);
/// Three 12 px lights, 8 px apart, as `AppKit` spaces them.
const LIGHTS_WIDTH: Pixels = px(52.);
/// From the last light to the workspace button.
const LIGHTS_GAP: Pixels = px(10.);
/// The size of `AppKit`'s button frame around each 12 px light.
const TRAFFIC_LIGHT_FRAME: Pixels = px(14.);
/// The workspace button's padding before its mark.
const WORKSPACE_PADDING: Pixels = px(7.);
/// Space above the first place, inside the scrolling list.
const LIST_TOP: Pixels = px(2.);
/// Group name for the whole sidebar, whose hover shows the new folder button.
const SIDEBAR_GROUP: &str = "sidebar";

/// Where `AppKit` should put the close button: on the titlebar row's centre line, its light
/// 16 px in from the panel's edge.
pub fn traffic_light_position() -> Point<Pixels> {
    let frame_inset = (TRAFFIC_LIGHT_FRAME - px(12.)) / 2.;
    let left = MARGIN + BORDER + LIGHTS_INSET - frame_inset;
    let top = MARGIN + BORDER + (TITLEBAR_HEIGHT - TRAFFIC_LIGHT_FRAME) / 2.;
    gpui::point(left, top)
}

/// The workspace switcher's left edge in the window. The marks in its list line up under
/// the workspace button's.
pub(super) fn workspace_menu_left(mark_inset: Pixels) -> Pixels {
    let button_left = MARGIN + BORDER + LIGHTS_INSET + LIGHTS_WIDTH + LIGHTS_GAP;
    button_left + WORKSPACE_PADDING - mark_inset
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
    cloud: Entity<Cloud>,
    places: PlaceRows,
    /// Which parents are collapsed, and the folder rows that leaves.
    outline: Outline,
    drop: SidebarDrop,
    /// The folder whose menu is open from its ⋯ button, which stays shown meanwhile.
    folder_menu: Option<FolderId>,
    status: StatusCard,
    identity: Identity,
    account_menu: AccountMenu,
    /// The workspace switcher is open, so its button stays lit.
    workspaces_open: bool,
    /// What the sidebar last drew from the library, the organizer and the cloud.
    shown: Shown,
    _subscriptions: Vec<Subscription>,
}

struct Shown {
    library: LibraryShown,
    organizer: AiStatus,
    cloud: CloudShown,
}

impl EventEmitter<SidebarEvent> for SidebarView {}

impl SidebarView {
    pub(super) fn new(library: Entity<Library>, cx: &mut Context<'_, SidebarView>) -> SidebarView {
        let organizer = Organizer::entity(cx);
        let cloud = Cloud::entity(cx);
        let shown = Shown {
            library: LibraryShown::of(library.read(cx)),
            organizer: organizer.read(cx).status().clone(),
            cloud: CloudShown::of(cloud.read(cx), SettingsFile::get(cx), false),
        };
        // Each update redraws the sidebar only when it changes what the sidebar shows.
        // The batch and its progress come with the library, and the sync time and the
        // changes waiting with the cloud.
        let subscriptions = vec![
            cx.observe(&library, |this, library, cx| {
                let now = LibraryShown::of(library.read(cx));
                if shown::update(&mut this.shown.library, now) {
                    cx.notify();
                }
            }),
            cx.observe(&organizer, |this, organizer, cx| {
                let now = organizer.read(cx).status().clone();
                if shown::update(&mut this.shown.organizer, now) {
                    cx.notify();
                }
            }),
            cx.observe(&cloud, |this, cloud, cx| {
                let menu_open = this.account_menu.is_open();
                let now = CloudShown::of(cloud.read(cx), SettingsFile::get(cx), menu_open);
                if shown::update(&mut this.shown.cloud, now) {
                    cx.notify();
                }
            }),
        ];
        SidebarView {
            library,
            organizer,
            cloud,
            places: PlaceRows::default(),
            outline: Outline::load(cx),
            drop: SidebarDrop::default(),
            folder_menu: None,
            status: StatusCard::default(),
            identity: Identity::default(),
            account_menu: AccountMenu::default(),
            workspaces_open: false,
            shown,
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
        let current = self.library.read(cx).place();
        div()
            .id("sidebar")
            .group(SIDEBAR_GROUP)
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .rounded(PANEL_RADIUS)
            .bg(theme.panel)
            .border_1()
            .border_color(theme.hairline)
            .shadow_sm()
            .text_color(theme.text)
            .child(self.titlebar(cx))
            .child(
                div()
                    .id("sidebar-list")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.drop.scroll)
                    .on_scroll(cx.listener(|this, _, _, cx| {
                        if cx.has_active_drag() {
                            this.drop.end();
                            cx.notify();
                        }
                    }))
                    .pt(LIST_TOP)
                    .px(INSET)
                    .child(self.place_list(current, cx))
                    .child(self.folder_header(cx))
                    .child(self.folder_list(current, cx)),
            )
            .children(self.status_card(cx))
            .child(self.footer(cx))
            .children(self.account_menu(cx))
    }
}
