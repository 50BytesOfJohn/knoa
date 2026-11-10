//! What the library area says when a place or a search has no bookmarks.

use gpui::{Context, IntoElement, ParentElement as _, Styled as _, div, px};
use knoa_core::Place;

use crate::ui::{
    components::icon,
    main::MainView,
    mascot::{Mood, mascot},
    theme::Theme,
};

impl MainView {
    pub(super) fn empty_state(&self, cx: &mut Context<'_, MainView>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let library = self.library.read(cx);
        let caught_up = library.search().is_empty() && library.place() == Place::NotOrganized;
        let (title, body) = if !library.search().is_empty() {
            (
                "Nothing matches",
                "Try fewer words, or filters like kind:movie, site:github.com or #tag.".to_owned(),
            )
        } else if library.place() == Place::Trash {
            (
                "Trash is empty",
                "Bookmarks you delete wait here for 30 days.".to_owned(),
            )
        } else if library.place() == Place::NotOrganized {
            (
                "All caught up",
                "Every bookmark here has been organized.".to_owned(),
            )
        } else if library.place() == Place::Everything {
            let synced = library.workspace().is_some_and(|w| !w.browsers.is_empty());
            if synced {
                (
                    "Nothing here yet",
                    "Bookmark something in your browser and it lands here a moment later."
                        .to_owned(),
                )
            } else {
                (
                    "Nothing here yet",
                    "Add a link with ⌘N, or pick a browser for this workspace from the workspace menu."
                        .to_owned(),
                )
            }
        } else {
            (
                "Nothing here yet",
                "Bookmarks show up here as you add them.".to_owned(),
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
