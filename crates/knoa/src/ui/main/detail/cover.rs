//! The detail panel's cover image, and how a thumbnail refresh is going.

use gpui::{
    App, Div, ObjectFit, ParentElement as _, Pixels, Styled as _, StyledImage as _, div, img, px,
};
use knoa_core::{BookmarkId, Image};

use crate::{
    services::thumbnails::{Refresh, Thumbnails},
    ui::{
        components::icon,
        theme::{DETAIL_WIDTH, PANEL_RADIUS, Theme},
    },
};

/// Taller images (posters, book covers) are shown smaller so the text still starts on screen.
const MAX_COVER_HEIGHT: f32 = 300.;

/// A small pill while the bookmark's thumbnail is being fetched again, and for a moment
/// after a refresh that found nothing. A found image speaks for itself. On the cover it
/// takes the cards' dark badge look, so it reads on any picture.
pub(super) fn refresh_status(id: BookmarkId, on_image: bool, cx: &App) -> Option<Div> {
    let (glyph, text) = match Thumbnails::entity(cx).read(cx).refresh_of(id)? {
        Refresh::Running => ("refresh-cw", "Refreshing thumbnail…"),
        Refresh::NoImage => ("image-off", "No thumbnail on the page"),
        Refresh::Failed => ("wifi-off", "Couldn't reach the page"),
    };
    let theme = Theme::get(cx);
    let (bg, fg) = if on_image {
        (gpui::black().opacity(0.55), gpui::white())
    } else {
        (theme.soft, theme.muted)
    };
    Some(
        div()
            .flex()
            .items_center()
            .gap_1p5()
            .h(px(24.))
            .px_2p5()
            .rounded_full()
            .bg(bg)
            .text_xs()
            .text_color(fg)
            .child(icon(glyph, px(12.), fg))
            .child(text),
    )
}

/// The image under the panel's top edge, always whole: the cards crop to keep the grid
/// calm, so the detail panel is where a poster or logo shows in full.
pub(super) fn image(image: &Image, theme: &Theme) -> Div {
    let natural = f32::from(DETAIL_WIDTH) * image.aspect_ratio();
    let full_bleed = natural <= MAX_COVER_HEIGHT;
    let height = natural.min(MAX_COVER_HEIGHT);
    // Too tall to show full-bleed: shown whole, inset and centered on the backdrop.
    let inset = if full_bleed { 0. } else { 12. };
    // Round to the panel's inner edge, inside its 1 px border.
    let radius: Pixels = PANEL_RADIUS - px(1.);
    // `img` takes the picture's aspect ratio unless one is set, and layout follows that
    // ratio over an explicit height, so the box's own ratio is set: inside the panel's
    // side borders and above the box's bottom hairline.
    let inner = (f32::from(DETAIL_WIDTH) - 2., height - 1. - inset * 2.);
    let picture = img(image.path.clone())
        .w_full()
        .h(px(inner.1))
        .aspect_ratio(inner.0 / inner.1);
    let picture = if full_bleed {
        picture.rounded_t(radius).object_fit(ObjectFit::Fill)
    } else {
        picture.rounded(px(6.)).object_fit(ObjectFit::Contain)
    };
    div()
        .h(px(height))
        .w_full()
        .py(px(inset))
        .overflow_hidden()
        .rounded_t(radius)
        .bg(theme.soft)
        .border_b_1()
        .border_color(theme.hairline)
        .child(picture)
}
