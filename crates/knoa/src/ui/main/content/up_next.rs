//! The Up next strip above the cards: bookmarks marked to read and watch next.

use gpui::{
    ClickEvent, Context, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div, px,
};
use knoa_core::{Bookmark, Place};

use crate::{
    state::settings::SettingsFile,
    ui::{
        components::{icon, site_mark},
        main::{MainView, bookmark_menu::with_bookmark_menu},
        theme::{PANEL_RADIUS, ROW_RADIUS, Theme},
    },
};

pub(super) const UP_NEXT_HEIGHT: f32 = 92.;

impl MainView {
    pub(super) fn up_next_items(&self, cx: &Context<'_, MainView>) -> Vec<Bookmark> {
        let library = self.library.read(cx);
        if library.place() != Place::Everything
            || !library.search().is_empty()
            || !SettingsFile::get(cx).show_up_next
        {
            return Vec::new();
        }
        library
            .results()
            .iter()
            .filter(|b| b.up_next)
            .take(12)
            .cloned()
            .collect()
    }

    pub(super) fn up_next_strip(
        &self,
        items: &[Bookmark],
        width: f32,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let mut row = div().id("up-next-row").flex().gap_2().overflow_x_scroll();
        for bookmark in items {
            let id = bookmark.id;
            let item = div()
                .id(("up-next", id.0 as usize))
                .flex_none()
                .w(px(200.))
                .h(px(44.))
                .px_2()
                .flex()
                .items_center()
                .gap_2()
                .rounded(ROW_RADIUS)
                .bg(theme.soft)
                .cursor_pointer()
                .hover(|s| s.bg(theme.hover))
                .child(site_mark(&bookmark.site, px(20.), cx))
                .child(div().text_xs().line_clamp(2).child(bookmark.title.clone()))
                .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                    this.click_card(id, event, window, cx);
                }))
                .on_aux_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                    this.click_card(id, event, window, cx);
                }));
            row = row.child(with_bookmark_menu(item, id, cx));
        }
        div()
            .absolute()
            .top_0()
            .left_0()
            .w(px(width))
            .h(px(UP_NEXT_HEIGHT))
            .p_3()
            .rounded(PANEL_RADIUS)
            .bg(theme.panel)
            .border_1()
            .border_color(theme.hairline)
            .child(
                div()
                    .flex()
                    .items_center()
                    .mb_2()
                    .text_xs()
                    .gap_1p5()
                    .child(icon("bookmark", px(13.), theme.accent))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Up next"))
                    .child(
                        div()
                            .text_color(theme.muted)
                            .child(format!("· {} to read and watch", items.len())),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .id("hide-up-next")
                            .cursor_pointer()
                            .child(icon("x", px(14.), theme.faint))
                            .on_click(cx.listener(|_, _, _, cx| {
                                SettingsFile::update(cx, |s| s.show_up_next = false);
                                cx.notify();
                            })),
                    ),
            )
            .child(row)
    }
}
