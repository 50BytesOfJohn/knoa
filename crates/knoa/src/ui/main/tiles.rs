//! Gallery tiles: a bookmark's picture alone, with its title and site laid over it on
//! hover or selection. A tile's height follows from the stored image's size (ADR 0007).

use gpui::{
    App, Div, FontWeight, InteractiveElement as _, ObjectFit, ParentElement as _, Styled as _,
    StyledImage as _, div, img, linear_color_stop, linear_gradient, prelude::FluentBuilder as _,
    px,
};
use knoa_core::Bookmark;

use crate::ui::{
    components::icon,
    main::cards::GROUP,
    theme::{CARD_RADIUS, Theme},
};

/// Height over width. Wider panoramas and taller strips are cropped to these, so one
/// picture can't turn into a sliver or fill the whole column.
const FLATTEST: f32 = 0.4;
const TALLEST: f32 = 2.2;
/// A tile with no picture keeps a calm, photo-like shape.
const PLACEHOLDER_RATIO: f32 = 0.75;

/// The tile's height at `width`.
pub fn height(bookmark: &Bookmark, width: f32) -> f32 {
    let ratio = bookmark.image.as_ref().map_or(PLACEHOLDER_RATIO, |image| {
        image.aspect_ratio().clamp(FLATTEST, TALLEST)
    });
    width * ratio
}

pub fn render(bookmark: &Bookmark, width: f32, selected: bool, cx: &App) -> Div {
    let theme = Theme::get(cx);
    let height = height(bookmark, width);
    let tile = match &bookmark.image {
        Some(image) => div()
            .size_full()
            .rounded(CARD_RADIUS)
            .overflow_hidden()
            .bg(theme.soft)
            // `img` takes the picture's own ratio unless one is set, so the clamped box's
            // ratio is given for the crop to hold.
            .child(
                img(image.path.clone())
                    .w_full()
                    .h(px(height))
                    .aspect_ratio(width / height)
                    .rounded(CARD_RADIUS)
                    .object_fit(ObjectFit::Cover),
            )
            .child(caption(bookmark, selected)),
        None => placeholder(bookmark, theme),
    };
    tile.relative().when(selected, |s| {
        s.child(
            div()
                .absolute()
                .inset_0()
                .rounded(CARD_RADIUS)
                .border_2()
                .border_color(theme.accent),
        )
    })
}

/// Title and site on a dark fade at the bottom, shown while hovered or selected.
fn caption(bookmark: &Bookmark, selected: bool) -> Div {
    let fade = linear_gradient(
        0.,
        linear_color_stop(gpui::black().opacity(0.7), 0.),
        linear_color_stop(gpui::black().opacity(0.), 1.),
    );
    div()
        .absolute()
        .left_0()
        .right_0()
        .bottom_0()
        .px_3()
        .pt_8()
        .pb_2p5()
        .rounded_b(CARD_RADIUS)
        .bg(fade)
        .text_color(gpui::white())
        .when(!selected, |s| {
            s.invisible().group_hover(GROUP, gpui::Styled::visible)
        })
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .truncate()
                .child(bookmark.title.clone()),
        )
        .child(
            div()
                .text_xs()
                .text_color(gpui::white().opacity(0.75))
                .truncate()
                .child(bookmark.site.clone()),
        )
}

/// A picture Knoa couldn't keep (a format it can't decode, a file too large) is named
/// instead of drawn as a broken image.
fn placeholder(bookmark: &Bookmark, theme: &Theme) -> Div {
    let format = bookmark
        .fields
        .get("format")
        .and_then(|value| value.as_str())
        .map(str::to_owned);
    div()
        .size_full()
        .p_4()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .rounded(CARD_RADIUS)
        .border_1()
        .border_dashed()
        .border_color(theme.faint.opacity(0.6))
        .bg(theme.card.opacity(0.6))
        .text_color(theme.muted)
        .child(icon("image-off", px(20.), theme.faint))
        .child(
            div()
                .max_w_full()
                .text_sm()
                .text_center()
                .line_clamp(2)
                .child(bookmark.title.clone()),
        )
        .when_some(format, |s, format| {
            s.child(div().text_xs().text_color(theme.faint).child(format))
        })
}
