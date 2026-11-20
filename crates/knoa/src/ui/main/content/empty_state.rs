//! What the library area says when a place or a search has no bookmarks.

use gpui::{Context, IntoElement, ParentElement as _, Styled as _, div, px};
use knoa_core::Place;

use crate::{
    i18n::t,
    ui::{
        components::icon,
        main::MainView,
        mascot::{Mood, mascot},
        theme::Theme,
    },
};

impl MainView {
    pub(in crate::ui::main) fn empty_state(
        &self,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx);
        let library = self.library.read(cx);
        let caught_up = library.search().is_empty() && library.place() == Place::NotOrganized;
        let (title, body) = if !library.search().is_empty() && library.place() == Place::Events {
            (
                t!("main-empty-no-events-match"),
                t!("main-empty-no-events-match-body"),
            )
        } else if !library.search().is_empty() {
            (
                t!("main-empty-nothing-matches"),
                t!("main-empty-nothing-matches-body"),
            )
        } else if library.place() == Place::Trash {
            (t!("main-empty-trash"), t!("main-empty-trash-body"))
        } else if library.place() == Place::NotOrganized {
            (t!("main-empty-caught-up"), t!("main-empty-caught-up-body"))
        } else if library.place() == Place::Gallery {
            (
                t!("main-empty-no-pictures"),
                t!("main-empty-no-pictures-body"),
            )
        } else if library.place() == Place::Events {
            (t!("main-empty-no-events"), t!("main-empty-no-events-body"))
        } else if library.place() == Place::Everything {
            let synced = library.workspace().is_some_and(|w| !w.browsers.is_empty());
            if synced {
                (
                    t!("main-empty-nothing-yet"),
                    t!("main-empty-nothing-yet-synced-body"),
                )
            } else {
                (
                    t!("main-empty-nothing-yet"),
                    t!("main-empty-nothing-yet-unsynced-body"),
                )
            }
        } else {
            (
                t!("main-empty-nothing-yet"),
                t!("main-empty-nothing-yet-body"),
            )
        };
        div()
            .pt(px(80.))
            .flex()
            .flex_col()
            .items_center()
            .gap_2()
            .text_color(theme.muted)
            .child(if caught_up {
                mascot("caught-up", px(56.), Mood::Happy, theme.ai, theme.canvas).into_any_element()
            } else {
                icon("telescope", px(28.), theme.faint).into_any_element()
            })
            .child(
                div()
                    .font_family(crate::ui::theme::SERIF)
                    .text_xl()
                    .text_color(theme.text)
                    .child(title),
            )
            .child(div().text_sm().max_w(px(360.)).text_center().child(body))
    }
}
