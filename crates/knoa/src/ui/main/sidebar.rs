//! The floating sidebar: the titlebar with the workspace and add-link buttons, places,
//! folders and the footer with the wordmark, the AI indicator and settings.

use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    Pixels, SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_component::tooltip::Tooltip;
use knoa_core::Place;

mod footer;

use super::{MainView, NewLink, Overlay};
use crate::ui::{
    components::{icon, workspace_mark},
    mascot::ai_mark,
    theme::{MARGIN, PANEL_RADIUS, SIDEBAR_WIDTH, Theme},
};

/// What leads a place's row: a Lucide icon, or the AI's dot for places about the AI.
#[derive(Clone, Copy)]
enum PlaceMark {
    Icon(&'static str),
    Ai,
}

/// The sidebar's top row, shared with the traffic lights. With the sidebar's 1 px border its
/// middle lines up with theirs, which `traffic_light_position` in `app.rs` places.
pub(super) const TITLEBAR_HEIGHT: Pixels = px(40.);
const TITLEBAR_PADDING: Pixels = px(8.);
const TITLEBAR_GAP: Pixels = px(4.);
const TITLEBAR_BUTTON: Pixels = px(26.);
const WORKSPACE_BUTTON_WIDTH: Pixels = px(40.);
const WORKSPACE_MARK: Pixels = px(20.);

/// The workspace switcher's left edge in the window. It sits under the workspace button,
/// shifted left by the menu's and its rows' padding so the marks in the list line up under
/// the button's.
pub(super) fn workspace_menu_left() -> Pixels {
    let border = px(1.);
    let button_left = MARGIN + SIDEBAR_WIDTH
        - border
        - TITLEBAR_PADDING
        - TITLEBAR_BUTTON
        - TITLEBAR_GAP
        - WORKSPACE_BUTTON_WIDTH;
    button_left - px(16.)
}

impl MainView {
    pub(super) fn render_sidebar(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let library = self.library.read(cx);
        let counts = library.counts();
        let current = library.place();
        let folders = library.folders().to_vec();
        let (name, color) = library
            .workspace()
            .map(|w| (w.name.clone(), w.color))
            .unwrap_or_default();

        let list = self.places(counts, current, cx);

        let mut folder_list = div().flex().flex_col().gap_0p5().px_2();
        for (index, folder) in folders.iter().enumerate() {
            let shortcut = (index < 9).then(|| format!("⌘{}", index + 1));
            let color = theme.folder_color(folder.color.0);
            let row = self.place_row(
                Place::Folder(folder.id),
                current,
                &folder.name,
                PlaceMark::Icon("folder"),
                folder.count,
                Some((color, shortcut)),
                cx,
            );
            let id = folder.id;
            folder_list = folder_list.child(
                div()
                    .id(("folder-row", id.0 as usize))
                    .child(row)
                    .on_mouse_down(
                        gpui::MouseButton::Right,
                        cx.listener(move |this, _, window, cx| {
                            this.edit_folder(Some(id), window, cx);
                        }),
                    ),
            );
        }
        folder_list = folder_list.child(
            div()
                .id("new-folder")
                .flex()
                .items_center()
                .gap_2()
                .h(px(30.))
                .px_2()
                .rounded(px(9.))
                .text_sm()
                .text_color(theme.muted)
                .cursor_pointer()
                .hover(|s| s.bg(theme.hover))
                .child(icon("plus", px(15.), theme.muted))
                .child("New folder")
                .on_click(cx.listener(|this, _, window, cx| this.edit_folder(None, window, cx))),
        );

        div()
            .absolute()
            .top(MARGIN)
            .left(MARGIN)
            .bottom(MARGIN)
            .w(SIDEBAR_WIDTH)
            .flex()
            .flex_col()
            .rounded(PANEL_RADIUS)
            .bg(theme.panel)
            .border_1()
            .border_color(theme.hairline)
            .shadow_sm()
            .child(
                div()
                    .h(TITLEBAR_HEIGHT)
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap(TITLEBAR_GAP)
                    .pr(TITLEBAR_PADDING)
                    .child(self.workspace_button(&name, color, cx))
                    .child(self.add_link_button(cx)),
            )
            .child(div().mt_2().child(list))
            .child(
                div()
                    .id("folders")
                    .mt_4()
                    .flex_1()
                    .overflow_y_scroll()
                    .child(
                        div()
                            .px_4()
                            .pb_1()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.faint)
                            .child("Folders"),
                    )
                    .child(folder_list),
            )
            .child(self.footer(counts.organizing, cx))
    }

    fn places(
        &self,
        counts: knoa_store::Counts,
        current: Place,
        cx: &mut Context<'_, MainView>,
    ) -> gpui::Div {
        let places = [
            (
                Place::Everything,
                "Everything",
                PlaceMark::Icon("layers"),
                counts.everything,
            ),
            (
                Place::UpNext,
                "Up next",
                PlaceMark::Icon("bookmark"),
                counts.up_next,
            ),
            (
                Place::Unsorted,
                "Unsorted",
                PlaceMark::Icon("inbox"),
                counts.unsorted,
            ),
            (
                Place::NotOrganized,
                "Not organized",
                PlaceMark::Ai,
                counts.not_organized,
            ),
            (
                Place::SetAside,
                "Set aside",
                PlaceMark::Icon("archive"),
                counts.set_aside,
            ),
            (
                Place::Trash,
                "Trash",
                PlaceMark::Icon("trash"),
                counts.trash,
            ),
        ];
        let mut list = div().flex().flex_col().gap_0p5().px_2();
        for (place, label, mark, count) in places {
            // Set aside and Trash only show while they hold something.
            let hideable = matches!(place, Place::SetAside | Place::Trash);
            if hideable && count == 0 && current != place {
                continue;
            }
            list = list.child(self.place_row(place, current, label, mark, count, None, cx));
        }
        list
    }

    /// Only the workspace's mark: most people keep one workspace, so its name would say
    /// little. The name shows on hover and in the switcher.
    fn workspace_button(
        &self,
        name: &str,
        color: u8,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx);
        let open = matches!(self.overlay, Some(Overlay::Workspaces));
        let tooltip = SharedString::from(name.to_owned());
        let (bg, fg, border) = (theme.panel, theme.text, theme.hairline);
        div()
            .id("workspace-button")
            .w(WORKSPACE_BUTTON_WIDTH)
            .h(TITLEBAR_BUTTON)
            .pl_1()
            .flex()
            .items_center()
            .gap(px(3.))
            .rounded(px(8.))
            .cursor_pointer()
            .when(open, |s| s.bg(theme.hover))
            .hover(|s| s.bg(theme.hover))
            .tooltip(move |window, cx| {
                Tooltip::new(tooltip.clone())
                    .bg(bg)
                    .text_color(fg)
                    .border_color(border)
                    .build(window, cx)
            })
            .on_click(cx.listener(|this, _, _, cx| this.open_overlay(Overlay::Workspaces, cx)))
            .child(workspace_mark(name, color, WORKSPACE_MARK, cx))
            .child(icon("chevron-down", px(10.), theme.faint))
    }

    fn add_link_button(&self, cx: &mut Context<'_, MainView>) -> impl IntoElement {
        let theme = Theme::get(cx);
        div()
            .id("add-link")
            .size(TITLEBAR_BUTTON)
            .rounded(px(8.))
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.bg(theme.hover))
            .child(icon("plus", px(16.), theme.muted))
            .on_click(cx.listener(|this, _, window, cx| {
                this.new_link(&NewLink, window, cx);
            }))
    }

    #[allow(clippy::too_many_arguments)]
    fn place_row(
        &self,
        place: Place,
        current: Place,
        label: &str,
        mark: PlaceMark,
        count: usize,
        folder: Option<(gpui::Hsla, Option<String>)>,
        cx: &mut Context<'_, MainView>,
    ) -> AnyElement {
        let theme = Theme::get(cx);
        let selected = place == current;
        let icon_color = match &folder {
            Some((color, _)) => *color,
            None if selected => theme.accent,
            None => theme.muted,
        };
        let badge = match place {
            // Not organized only shows a count while the AI has work waiting.
            Place::NotOrganized if count == 0 => None,
            _ => Some(count),
        };
        let id = SharedString::from(format!("place-{label}"));
        div()
            .id(id)
            .flex()
            .items_center()
            .gap_2()
            .h(px(30.))
            .px_2()
            .rounded(px(9.))
            .text_sm()
            .cursor_pointer()
            .when(selected, |s| {
                s.bg(theme.hover).font_weight(FontWeight::MEDIUM)
            })
            .hover(|s| s.bg(theme.hover))
            .child(match mark {
                PlaceMark::Icon(name) => icon(name, px(15.), icon_color).into_any_element(),
                PlaceMark::Ai => ai_mark(px(15.), theme.ai).into_any_element(),
            })
            .child(div().flex_1().truncate().child(label.to_owned()))
            .when_some(folder.and_then(|(_, key)| key), |s, key| {
                s.child(div().text_xs().text_color(theme.faint).child(key))
            })
            .when_some(badge.filter(|n| *n > 0), |s, n| {
                s.child(div().text_xs().text_color(theme.muted).child(n.to_string()))
            })
            .on_click(cx.listener(move |this, _, window, cx| this.set_place(place, window, cx)))
            .into_any_element()
    }
}
