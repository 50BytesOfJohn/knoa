//! The shape every sidebar row shares, places and folders alike: its height, padding,
//! icon column and count.

use gpui::{
    Div, FontFeatures, FontWeight, InteractiveElement as _, ParentElement as _, Pixels,
    SharedString, Styled as _, div, prelude::FluentBuilder as _, px,
};

use crate::ui::theme::{ROW_RADIUS, Theme};

/// The sidebar's rows are denser than the shared `ROW_HEIGHT`, which other lists keep.
pub(super) const ROW_HEIGHT: Pixels = px(30.);
pub(super) const ROW_GAP: Pixels = px(1.);
pub(super) const PADDING_LEFT: Pixels = px(8.);
pub(super) const PADDING_RIGHT: Pixels = px(10.);
pub(super) const ICON_GAP: Pixels = px(10.);
pub(super) const ICON: Pixels = px(16.);
/// Wide enough for two digits, so short counts line up at the right.
const COUNT_MIN_WIDTH: Pixels = px(20.);

/// The row's frame: a hover fill, and the selected fill and weight when chosen.
pub(super) fn row_frame(selected: bool, theme: &Theme) -> Div {
    let hover = theme.hover;
    div()
        .relative()
        .flex()
        .flex_none()
        .items_center()
        .gap(ICON_GAP)
        .h(ROW_HEIGHT)
        .pl(PADDING_LEFT)
        .pr(PADDING_RIGHT)
        .rounded(ROW_RADIUS)
        .text_size(px(13.))
        .cursor_pointer()
        .when(selected, |s| {
            s.bg(theme.selected_row()).font_weight(FontWeight::MEDIUM)
        })
        .when(!selected, |s| s.hover(|s| s.bg(hover)))
}

/// The label, cut short with an ellipsis.
pub(super) fn label(text: SharedString) -> Div {
    div().flex_1().min_w_0().truncate().child(text)
}

/// The count at the row's right end, in figures of one width so counts align. Empty
/// when there is nothing to count.
pub(super) fn count(text: SharedString, theme: &Theme) -> Div {
    div()
        .flex_none()
        .min_w(COUNT_MIN_WIDTH)
        .text_right()
        .text_size(px(12.))
        .font_features(tabular_figures())
        .text_color(theme.muted)
        .child(text)
}

fn tabular_figures() -> FontFeatures {
    thread_local! {
        static TNUM: FontFeatures = FontFeatures(std::sync::Arc::new(vec![("tnum".into(), 1)]));
    }
    TNUM.with(Clone::clone)
}
