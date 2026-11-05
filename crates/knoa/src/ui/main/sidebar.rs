//! The floating sidebar: the titlebar with the workspace button, places, folders and the
//! footer with the AI indicator and settings.

use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    Pixels, SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};
use knoa_core::Place;

use super::{MainView, NewLink, OpenSettings, Overlay};
use crate::{
    services::organizer::AiStatus,
    ui::{
        components::{icon, workspace_mark},
        theme::{MARGIN, PANEL_RADIUS, SIDEBAR_WIDTH, Theme},
        wordmark::wordmark,
    },
};

/// The sidebar's top row, shared with the traffic lights. Its middle lines up with theirs.
pub(super) const TITLEBAR_HEIGHT: Pixels = px(42.);
/// Where the workspace button starts in the sidebar: 10 px past the zoom button. Moves with
/// `traffic_light_position` in `app.rs`.
pub(super) const WORKSPACE_BUTTON_LEFT: Pixels = px(84.);

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
                "folder",
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
                    .pl(WORKSPACE_BUTTON_LEFT)
                    .pr_2()
                    .child(self.workspace_button(&name, color, cx)),
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
            (Place::Everything, "Everything", "layers", counts.everything),
            (Place::UpNext, "Up next", "bookmark", counts.up_next),
            (Place::Unsorted, "Unsorted", "inbox", counts.unsorted),
            (
                Place::NotOrganized,
                "Not organized",
                "sparkles",
                counts.not_organized,
            ),
            (Place::SetAside, "Set aside", "archive", counts.set_aside),
            (Place::Trash, "Trash", "trash", counts.trash),
        ];
        let mut list = div().flex().flex_col().gap_0p5().px_2();
        for (place, label, icon_name, count) in places {
            // Set aside and Trash only show while they hold something.
            let hideable = matches!(place, Place::SetAside | Place::Trash);
            if hideable && count == 0 && current != place {
                continue;
            }
            list = list.child(self.place_row(place, current, label, icon_name, count, None, cx));
        }
        list
    }

    fn workspace_button(
        &self,
        name: &str,
        color: u8,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx);
        let open = matches!(self.overlay, Some(Overlay::Workspaces));
        div()
            .id("workspace-button")
            .min_w_0()
            .h(px(26.))
            .pl_1()
            .pr_1p5()
            .flex()
            .items_center()
            .gap_1p5()
            .rounded(px(8.))
            .cursor_pointer()
            .when(open, |s| s.bg(theme.hover))
            .hover(|s| s.bg(theme.hover))
            .on_click(cx.listener(|this, _, _, cx| this.open_overlay(Overlay::Workspaces, cx)))
            .child(workspace_mark(name, color, px(18.), cx))
            .child(
                div()
                    .min_w_0()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .truncate()
                    .child(name.to_owned()),
            )
            .child(icon("chevron-down", px(12.), theme.faint))
    }

    #[allow(clippy::too_many_arguments)]
    fn place_row(
        &self,
        place: Place,
        current: Place,
        label: &str,
        icon_name: &'static str,
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
            .child(icon(icon_name, px(15.), icon_color))
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

    fn footer(&self, organizing: usize, cx: &mut Context<'_, MainView>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let status = self.organizer.read(cx).status().clone();
        let (ai_label, ai_color) = match &status {
            AiStatus::Off => ("AI off".to_owned(), theme.faint),
            AiStatus::Ready if organizing > 0 => (format!("✦ {organizing}"), theme.ai),
            AiStatus::Ready => ("✦".to_owned(), theme.ai),
            AiStatus::NeedsAttention(_) => ("✦ needs attention".to_owned(), theme.danger),
        };
        div()
            .flex_none()
            .h(px(48.))
            .px_4()
            .flex()
            .items_center()
            .gap_2()
            .border_t_1()
            .border_color(theme.hairline)
            .child(wordmark(px(20.), theme.text))
            .child(div().text_xs().text_color(ai_color).child(ai_label))
            .child(div().flex_1())
            .child(
                div()
                    .id("add-link")
                    .size(px(28.))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.hover))
                    .child(icon("plus", px(16.), theme.muted))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.new_link(&NewLink, window, cx);
                    })),
            )
            .child(
                div()
                    .id("settings")
                    .size(px(28.))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.hover))
                    .child(icon("settings", px(16.), theme.muted))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.open_settings(&OpenSettings, window, cx);
                    })),
            )
    }
}
