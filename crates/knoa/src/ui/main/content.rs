//! The library area: the Up next strip, date groups of masonry cards and the date rail.
//! Only cards that intersect the viewport are built each frame.

use std::sync::Arc;

use gpui::{
    AnyElement, ClickEvent, Context, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use knoa_core::{Bookmark, BookmarkId, Place};

use super::{
    MainView, cards,
    groups::{self, Group},
    masonry::{Layout, Spec},
    search::SEARCH_AREA_HEIGHT,
};
use crate::{
    state::settings::SettingsFile,
    ui::{
        components::{icon, site_mark},
        theme::{DETAIL_WIDTH, MARGIN, PANEL_RADIUS, SIDEBAR_WIDTH, Theme},
    },
};

const GAP: f32 = 14.;
const HEADER_HEIGHT: f32 = 38.;
const UP_NEXT_HEIGHT: f32 = 92.;
/// Cards built above and below the viewport, so fast scrolling doesn't show gaps.
const OVERSCAN: f32 = 600.;
const MIN_COLUMN_WIDTH: f32 = 190.;

/// The layout for the current results, recomputed only when its inputs change.
#[derive(Default)]
pub struct LayoutCache {
    key: Option<(usize, u32, usize, bool, bool, bool)>,
    pub groups: Vec<Group>,
    pub layout: Layout,
    pub results: Arc<Vec<Bookmark>>,
}

/// The horizontal span between the sidebar and the detail panel (or the window edge).
pub fn content_span(window: &Window, show_detail: bool) -> (f32, f32) {
    let width = f32::from(window.viewport_size().width);
    let left = f32::from(SIDEBAR_WIDTH + MARGIN * 2.);
    let right = if show_detail {
        width - f32::from(DETAIL_WIDTH + MARGIN * 2.)
    } else {
        width - f32::from(MARGIN)
    };
    (left, right.max(left + 200.))
}

impl MainView {
    pub(super) fn render_content(
        &mut self,
        show_detail: bool,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let (left, right) = content_span(window, show_detail);
        let rail_width = 36.;
        let width = right - left - rail_width - 8.;
        let up_next = self.up_next_items(cx);
        self.update_layout(width, show_detail, !up_next.is_empty(), cx);

        let viewport_height = f32::from(window.viewport_size().height);
        let scroll_top = -f32::from(self.scroll.offset().y);
        let results = self.layout.results.clone();
        let show_summaries = Self::show_summaries(cx);

        let mut canvas = div()
            .relative()
            .w(px(width))
            .h(px(self.layout.layout.height + f32::from(MARGIN)));
        if !up_next.is_empty() {
            canvas = canvas.child(self.up_next_strip(&up_next, width, cx));
        }
        for (group, y) in self.layout.groups.iter().zip(&self.layout.layout.headers) {
            canvas = canvas.child(
                div()
                    .absolute()
                    .top(px(*y + 8.))
                    .left_0()
                    .px_2p5()
                    .h(px(24.))
                    .flex()
                    .items_center()
                    .rounded_full()
                    .bg(theme.panel.opacity(0.7))
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(group.label.clone()),
            );
        }
        for placed in self.layout.layout.visible(
            scroll_top - OVERSCAN,
            scroll_top + viewport_height + OVERSCAN,
        ) {
            let Some(bookmark) = results.get(placed.index) else {
                continue;
            };
            let id = bookmark.id;
            let card = cards::render(bookmark, placed.width, show_summaries, cx);
            canvas = canvas.child(
                div()
                    .absolute()
                    .top(px(placed.y))
                    .left(px(placed.x))
                    .w(px(placed.width))
                    .child(
                        cards::interactive(placed.index, card, placed.height, &theme).on_click(
                            cx.listener(move |this, event: &ClickEvent, window, cx| {
                                this.click_card(id, event, window, cx);
                            }),
                        ),
                    ),
            );
        }

        let body: AnyElement = if results.is_empty() {
            self.empty_state(cx).into_any_element()
        } else {
            canvas.into_any_element()
        };
        div()
            .absolute()
            .top_0()
            .bottom_0()
            .left(px(left))
            .w(px(right - left))
            .child(
                div()
                    .id("cards")
                    .key_context("Cards")
                    .track_focus(&self.cards_focus)
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .on_scroll(cx.listener(|_, _, _, cx| cx.notify()))
                    .on_action(cx.listener(Self::move_left))
                    .on_action(cx.listener(Self::move_right))
                    .on_action(cx.listener(Self::move_up))
                    .on_action(cx.listener(Self::move_down))
                    .on_action(cx.listener(Self::open_selected))
                    .on_action(cx.listener(Self::toggle_detail))
                    .on_action(cx.listener(Self::toggle_up_next))
                    .on_action(cx.listener(Self::copy_link))
                    .on_action(cx.listener(Self::dismiss))
                    .child(
                        div()
                            .pt(px(SEARCH_AREA_HEIGHT + f32::from(MARGIN)))
                            .pb(MARGIN)
                            .child(body),
                    ),
            )
            .child(self.date_rail(viewport_height, cx))
    }

    fn update_layout(
        &mut self,
        width: f32,
        show_detail: bool,
        has_up_next: bool,
        cx: &mut Context<'_, MainView>,
    ) {
        let library = self.library.read(cx);
        let results = library.results();
        let show_summaries = Self::show_summaries(cx);
        // Relevance order has no dates to group by.
        let by_date = library.search().is_empty();
        let preferred = if show_detail { 3 } else { 4 };
        let columns = preferred
            .min(((width + GAP) / (MIN_COLUMN_WIDTH + GAP)).floor() as usize)
            .max(1);
        let key = (
            Arc::as_ptr(&results) as usize,
            width.round() as u32,
            columns,
            show_summaries,
            by_date,
            has_up_next,
        );
        if self.layout.key == Some(key) {
            return;
        }
        let groups = if by_date {
            let added: Vec<i64> = results.iter().map(|b| b.added_at).collect();
            groups::by_date(&added, &jiff::Zoned::now())
        } else {
            groups::single("Best matches", results.len())
        };
        let spec = Spec {
            width,
            columns,
            gap: GAP,
            header_height: HEADER_HEIGHT,
            top: if has_up_next {
                UP_NEXT_HEIGHT + GAP
            } else {
                0.
            },
        };
        let layout = Layout::masonry(&groups, &spec, |index, column_width| {
            cards::height(&results[index], column_width, show_summaries)
        });
        self.layout = LayoutCache {
            key: Some(key),
            groups,
            layout,
            results,
        };
    }

    fn click_card(
        &mut self,
        id: BookmarkId,
        event: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        self.cards_focus.focus(window, cx);
        if event.click_count() >= 2 {
            if let Some(bookmark) = self.library.read(cx).bookmark(id) {
                cx.open_url(&bookmark.url);
            }
            return;
        }
        self.select(id, cx);
    }

    fn up_next_items(&self, cx: &Context<'_, MainView>) -> Vec<Bookmark> {
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

    fn up_next_strip(
        &self,
        items: &[Bookmark],
        width: f32,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let mut row = div().id("up-next-row").flex().gap_2().overflow_x_scroll();
        for bookmark in items {
            let id = bookmark.id;
            row = row.child(
                div()
                    .id(("up-next", id.0 as usize))
                    .flex_none()
                    .w(px(200.))
                    .h(px(44.))
                    .px_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .rounded(px(10.))
                    .bg(theme.soft)
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.hover))
                    .child(site_mark(&bookmark.site, px(20.), cx))
                    .child(div().text_xs().line_clamp(2).child(bookmark.title.clone()))
                    .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                        this.click_card(id, event, window, cx);
                    })),
            );
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

    /// One line per date group; longer lines for busier periods. Hover shows the label.
    fn date_rail(&self, viewport_height: f32, cx: &mut Context<'_, MainView>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let groups = &self.layout.groups;
        let max = groups.iter().map(|g| g.len).max().unwrap_or(1) as f32;
        let scroll_top = -f32::from(self.scroll.offset().y);
        let current = self
            .layout
            .layout
            .headers
            .iter()
            .rposition(|y| *y <= scroll_top + 40.)
            .unwrap_or(0);
        let mut rail = div().flex().flex_col().items_end().gap_2();
        for (index, group) in groups.iter().enumerate().filter(|_| groups.len() > 1) {
            let length = 8. + 18. * (group.len as f32 / max).sqrt();
            let name = format!("rail-{index}");
            let target = self.layout.layout.headers[index];
            rail = rail.child(
                div()
                    .id(("rail", index))
                    .group(name.clone())
                    .relative()
                    .h(px(10.))
                    .w(px(30.))
                    .flex()
                    .items_center()
                    .justify_end()
                    .cursor_pointer()
                    .child(
                        div()
                            .h(px(2.))
                            .w(px(length))
                            .rounded_full()
                            .bg(if index == current {
                                theme.text
                            } else {
                                theme.faint.opacity(0.6)
                            })
                            .group_hover(name.clone(), |s| s.w(px(length + 8.)).bg(theme.text)),
                    )
                    .child(
                        div()
                            .absolute()
                            .right(px(36.))
                            .top(px(-7.))
                            .px_2()
                            .h(px(24.))
                            .flex()
                            .items_center()
                            .whitespace_nowrap()
                            .rounded_full()
                            .bg(theme.panel)
                            .border_1()
                            .border_color(theme.hairline)
                            .text_xs()
                            .invisible()
                            .group_hover(name, gpui::Styled::visible)
                            .child(format!("{} · {}", group.label, group.len)),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        let top = SEARCH_AREA_HEIGHT + f32::from(MARGIN) + target - 8.;
                        this.scroll
                            .set_offset(gpui::point(px(0.), px(-top.max(0.))));
                        cx.notify();
                    })),
            );
        }
        div()
            .absolute()
            .right_0()
            .top(px(viewport_height / 2. - 120.))
            .w(px(36.))
            .child(rail)
    }

    fn empty_state(&self, cx: &mut Context<'_, MainView>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let library = self.library.read(cx);
        let (title, body) = if !library.search().is_empty() {
            (
                "Nothing matches",
                "Try fewer words, or filters like kind:movie, site:github.com or #tag.".to_owned(),
            )
        } else if library.place() == Place::Everything {
            let browsers = &SettingsFile::get(cx).browsers;
            if browsers.is_empty() {
                (
                    "Your library is empty",
                    "Pick a browser in Settings and its bookmarks show up here.".to_owned(),
                )
            } else {
                (
                    "Your library is empty",
                    "Bookmark something in your browser and it lands here a moment later."
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
            .child(icon("telescope", px(28.), theme.faint))
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
