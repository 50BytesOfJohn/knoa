//! The library area: the Up next strip, date groups of masonry cards and the date rail.
//! Only cards that intersect the viewport are built each frame.

mod date_rail;
mod empty_state;
mod layout;
mod placed_card;
mod up_next;

use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use knoa_core::Place;

use super::{MainView, search::SEARCH_AREA_HEIGHT, trash::TRASH_BAR_HEIGHT};
use crate::ui::theme::{DETAIL_WIDTH, GUTTER, MARGIN, SIDEBAR_WIDTH, Theme};
pub(super) use layout::LayoutCache;
use up_next::UP_NEXT_HEIGHT;

const GAP: f32 = 16.;
/// Cards built above and below the viewport, so fast scrolling doesn't show gaps.
const OVERSCAN: f32 = 600.;

/// The horizontal span between the sidebar and the detail panel (or the window edge).
pub fn content_span(window: &Window, show_detail: bool) -> (f32, f32) {
    let width = f32::from(window.viewport_size().width);
    let left = f32::from(MARGIN + SIDEBAR_WIDTH + GUTTER);
    let right = if show_detail {
        width - f32::from(DETAIL_WIDTH + MARGIN + GUTTER)
    } else {
        width - f32::from(GUTTER)
    };
    (left, right.max(left + 200.))
}

/// Where the cards start: below the search pill, which floats as high as the sidebar.
pub(super) fn cards_top() -> f32 {
    SEARCH_AREA_HEIGHT + f32::from(MARGIN)
}

/// Room above the cards for the strip the place shows, if any.
fn strip_height(up_next: bool, trash_bar: bool) -> f32 {
    if up_next {
        UP_NEXT_HEIGHT + GAP
    } else if trash_bar {
        TRASH_BAR_HEIGHT + GAP
    } else {
        0.
    }
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
        let trash_bar = self.shows_trash_bar(cx);
        let top = strip_height(!up_next.is_empty(), trash_bar);
        self.update_layout(width, show_detail, top, cx);

        let viewport_height = f32::from(window.viewport_size().height);
        let scroll_top = -f32::from(self.scroll.offset().y);
        let results = self.layout.results.clone();
        let show_summaries = Self::show_summaries(cx);

        let mut canvas = div()
            .relative()
            .w(px(width))
            .h(px(self.layout.layout.height + f32::from(GUTTER)));
        if !up_next.is_empty() {
            canvas = canvas.child(self.up_next_strip(&up_next, width, cx));
        } else if trash_bar {
            canvas = canvas.child(self.trash_bar(width, cx));
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
            canvas = canvas.child(self.placed_card(bookmark, placed, show_summaries, &theme, cx));
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
                    .on_action(cx.listener(Self::open_selected_in_background))
                    .on_action(cx.listener(Self::toggle_detail))
                    .on_action(cx.listener(Self::toggle_up_next))
                    .on_action(cx.listener(Self::copy_link))
                    .on_action(cx.listener(Self::delete_selected))
                    .on_action(cx.listener(Self::dismiss))
                    .child(div().pt(px(cards_top())).pb(GUTTER).child(body)),
            )
            .child(self.date_rail(viewport_height, cx))
    }

    fn shows_trash_bar(&self, cx: &Context<'_, MainView>) -> bool {
        let library = self.library.read(cx);
        library.place() == Place::Trash && !library.results().is_empty()
    }
}
