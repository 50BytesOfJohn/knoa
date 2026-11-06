//! Library cards. Each card's height is computable without rendering it (ADR 0007): cards
//! keep to fixed regions (image, title, meta, summary) whose sizes follow from the data.

use gpui::{
    AnyElement, App, Div, FontWeight, Hsla, InteractiveElement as _, IntoElement, ObjectFit,
    ParentElement as _, Size, StatefulInteractiveElement as _, Styled as _, StyledImage as _, div,
    img, px, size,
};
use gpui_component::tooltip::Tooltip;
use knoa_core::{Bookmark, Kind, Status};

use crate::ui::{
    components::{chip, icon, kind_icon, site_mark},
    mascot::ai_mark,
    theme::{CARD_RADIUS, Theme},
};

const PADDING: f32 = 12.;
const TITLE_LINE: f32 = 19.;
const SUMMARY_LINE: f32 = 17.;
const META_HEIGHT: f32 = 18.;
const GAP: f32 = 6.;
/// Average glyph widths of the system font at the title and summary sizes.
const TITLE_CHAR: f32 = 7.1;
const SUMMARY_CHAR: f32 = 6.2;

/// How a bookmark is drawn, decided by what Knoa knows about it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// Only title and URL so far: compact and dashed.
    Pending,
    /// Repositories: text-only and dark.
    Code,
    Full,
}

fn shape(bookmark: &Bookmark) -> Shape {
    if bookmark.kind == Some(Kind::Repository) {
        Shape::Code
    } else if bookmark.image.is_none() && bookmark.summary.is_none() {
        Shape::Pending
    } else {
        Shape::Full
    }
}

/// The card's height at `width`.
pub fn height(bookmark: &Bookmark, width: f32, show_summary: bool) -> f32 {
    let inner = width - PADDING * 2.;
    let title = lines(
        &bookmark.title,
        inner,
        TITLE_CHAR,
        if shape(bookmark) == Shape::Pending {
            2
        } else {
            3
        },
    ) * TITLE_LINE;
    let summary = match (&bookmark.summary, show_summary) {
        (Some(text), true) => GAP + lines(text, inner, SUMMARY_CHAR, 3) * SUMMARY_LINE,
        _ => 0.,
    };
    match shape(bookmark) {
        Shape::Pending => PADDING * 2. + META_HEIGHT + GAP + title + GAP + 22.,
        Shape::Code => PADDING * 2. + TITLE_LINE + summary + GAP + META_HEIGHT,
        Shape::Full => {
            image_height(bookmark, width) + PADDING * 2. + title + GAP + META_HEIGHT + summary
        }
    }
}

fn image_height(bookmark: &Bookmark, width: f32) -> f32 {
    // Posters stay tall and banners short, within limits that keep the grid calm.
    bookmark
        .image
        .as_ref()
        .map_or(0., |image| width * image.aspect_ratio().clamp(0.45, 1.5))
}

fn lines(text: &str, width: f32, char_width: f32, max: usize) -> f32 {
    let per_line = (width / char_width).max(8.);
    let needed = (text.chars().count() as f32 / per_line).ceil().max(1.);
    needed.min(max as f32)
}

pub fn render(bookmark: &Bookmark, width: f32, show_summary: bool, cx: &App) -> Div {
    let theme = Theme::get(cx);
    match shape(bookmark) {
        Shape::Pending => pending(bookmark, width, theme, cx),
        Shape::Code => code(bookmark, width, show_summary, theme),
        Shape::Full => full(bookmark, width, show_summary, theme, cx),
    }
}

fn frame(theme: &Theme) -> Div {
    div()
        .size_full()
        .overflow_hidden()
        .rounded(CARD_RADIUS)
        .bg(theme.card)
        .border_1()
        .border_color(theme.hairline)
}

fn title(text: &str, max: usize, width: f32, color: Hsla) -> Div {
    let n = lines(text, width - PADDING * 2., TITLE_CHAR, max);
    div()
        .h(px(n * TITLE_LINE))
        .text_size(px(14.))
        .line_height(px(TITLE_LINE))
        .font_weight(FontWeight::MEDIUM)
        .text_color(color)
        .line_clamp(n as usize)
        .child(text.to_owned())
}

fn summary(text: &str, width: f32, color: Hsla) -> Div {
    let n = lines(text, width - PADDING * 2., SUMMARY_CHAR, 3);
    div()
        .mt(px(GAP))
        .h(px(n * SUMMARY_LINE))
        .text_size(px(12.5))
        .line_height(px(SUMMARY_LINE))
        .text_color(color)
        .line_clamp(n as usize)
        .child(text.to_owned())
}

fn meta(bookmark: &Bookmark, theme: &Theme, cx: &App) -> Div {
    let mut row = div()
        .h(px(META_HEIGHT))
        .flex()
        .items_center()
        .gap_1p5()
        .text_xs()
        .text_color(theme.muted)
        .overflow_hidden()
        .child(site_mark(&bookmark.site, px(14.), cx))
        // The site keeps its width; facts take what's left and get cut first.
        .child(div().flex_none().max_w(gpui::relative(0.6)).truncate().child(bookmark.site.clone()));
    let facts = bookmark
        .key_facts()
        .into_iter()
        .take(2)
        .collect::<Vec<_>>()
        .join(" · ");
    if !facts.is_empty() {
        row = row
            .child(div().text_color(theme.faint).child("·"))
            .child(div().flex_1().min_w_0().truncate().child(facts));
    }
    if bookmark.status.is_waiting() {
        row = row.child(div().flex_1()).child(ai_mark(px(12.), theme.ai));
    }
    row
}

fn pending(bookmark: &Bookmark, width: f32, theme: &Theme, cx: &App) -> Div {
    let (label, color, bg) = match bookmark.status {
        Status::Queued | Status::Organizing => ("Organizing…", theme.ai_text, theme.ai_soft),
        Status::Failed => ("Couldn't organize", theme.danger, theme.soft),
        _ => ("Not organized yet", theme.muted, theme.soft),
    };
    div()
        .size_full()
        .rounded(CARD_RADIUS)
        .border_1()
        .border_dashed()
        .border_color(theme.faint.opacity(0.6))
        .bg(theme.card.opacity(0.6))
        .p(px(PADDING))
        .child(
            div()
                .h(px(META_HEIGHT))
                .flex()
                .items_center()
                .gap_1p5()
                .text_xs()
                .text_color(theme.muted)
                .child(site_mark(&bookmark.site, px(14.), cx))
                .child(div().truncate().child(bookmark.site.clone())),
        )
        .child(
            div()
                .mt(px(GAP))
                .child(title(&bookmark.title, 2, width, theme.text)),
        )
        .child(div().mt(px(GAP)).flex().child(chip(label, bg, color)))
}

fn code(bookmark: &Bookmark, width: f32, show_summary: bool, theme: &Theme) -> Div {
    let name = match (bookmark.fields.get("owner"), bookmark.fields.get("name")) {
        (Some(owner), Some(name)) => format!(
            "{}/{}",
            owner.as_str().unwrap_or_default(),
            name.as_str().unwrap_or_default()
        ),
        _ => bookmark.title.clone(),
    };
    let facts = bookmark.key_facts().join(" · ");
    let mut card = div()
        .size_full()
        .rounded(CARD_RADIUS)
        .bg(theme.code_card)
        .p(px(PADDING))
        // owner/name has no spaces to wrap at, so it stays on one line.
        .child(
            div()
                .h(px(TITLE_LINE))
                .truncate()
                .font_family("Menlo")
                .text_size(px(13.))
                .text_color(theme.code_text)
                .child(name),
        );
    if let (Some(text), true) = (&bookmark.summary, show_summary) {
        card = card.child(summary(text, width, theme.code_text.opacity(0.7)));
    }
    card.child(
        div()
            .mt(px(GAP))
            .h(px(META_HEIGHT))
            .flex()
            .items_center()
            .gap_1p5()
            .text_xs()
            .text_color(theme.code_text.opacity(0.55))
            .child(icon("code", px(12.), theme.code_text.opacity(0.55)))
            .child(if facts.is_empty() {
                bookmark.site.clone()
            } else {
                facts
            }),
    )
}

fn full(bookmark: &Bookmark, width: f32, show_summary: bool, theme: &Theme, cx: &App) -> Div {
    let mut card = frame(theme);
    if let Some(image) = &bookmark.image {
        card = card.child(cover(
            bookmark,
            image.path.clone(),
            size(width, image_height(bookmark, width)),
            theme,
        ));
    }
    let mut body = div()
        .p(px(PADDING))
        .child(title(&bookmark.title, 3, width, theme.text))
        .child(div().mt(px(GAP)).child(meta(bookmark, theme, cx)));
    if let (Some(text), true) = (&bookmark.summary, show_summary) {
        body = body.child(summary(text, width, theme.muted));
    }
    card.child(body)
}

fn cover(
    bookmark: &Bookmark,
    path: std::path::PathBuf,
    card: Size<f32>,
    theme: &Theme,
) -> AnyElement {
    let badge = |name: &str| {
        div()
            .size(px(24.))
            .rounded_full()
            .bg(gpui::black().opacity(0.45))
            .flex()
            .items_center()
            .justify_center()
            .child(icon(name, px(13.), gpui::white()))
    };
    // GPUI clips overflow to a rectangle, so the card's rounding doesn't reach the image;
    // round it to the card's inner edge, inside its 1 px border.
    let radius = CARD_RADIUS - px(1.);
    // `img` takes the picture's aspect ratio unless one is set, and layout follows that
    // ratio over an explicit height, so a poster or banner outside the card's range would
    // spill out of its box. Give it the box's own ratio, inside the card's border.
    let height = card.height;
    let mut cover = div()
        .relative()
        .w_full()
        .h(px(height))
        .rounded_t(radius)
        .bg(theme.soft)
        .child(
            img(path)
                .w_full()
                .h(px(height))
                .aspect_ratio((card.width - 2.) / height)
                .rounded_t(radius)
                .object_fit(ObjectFit::Cover),
        )
        .child(
            div()
                .absolute()
                .top_2()
                .left_2()
                .child(badge(kind_icon(bookmark.kind))),
        );
    if bookmark.up_next {
        cover = cover.child(div().absolute().top_2().right_2().child(badge("bookmark")));
    }
    if bookmark.kind == Some(Kind::Video) {
        cover = cover.child(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .size(px(40.))
                        .rounded_full()
                        .bg(gpui::black().opacity(0.5))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(icon("play", px(18.), gpui::white())),
                ),
        );
    }
    cover.into_any_element()
}

const GROUP: &str = "card";

/// Wraps a card so it can be clicked, with the 2 px hover lift.
/// The card fills its layout slot, so a text-height estimate that's a little off shows as
/// padding instead of a gap in the grid.
pub fn interactive(id: usize, card: Div, height: f32, theme: &Theme) -> gpui::Stateful<Div> {
    let shadow = theme.card_hover_shadow;
    div()
        .id(("card", id))
        .group(GROUP)
        .relative()
        .h(px(height))
        .rounded(CARD_RADIUS)
        .cursor_pointer()
        .hover(move |s| {
            s.mt(px(-2.)).shadow(vec![gpui::BoxShadow {
                color: shadow,
                offset: gpui::point(px(0.), px(6.)),
                blur_radius: px(18.),
                spread_radius: px(0.),
                inset: false,
            }])
        })
        .child(card)
}

/// The button a card shows on hover, so opening takes one click; attach `on_click` and
/// add it to [`interactive`].
pub fn open_button(id: usize, theme: &Theme) -> gpui::Stateful<Div> {
    let (bg, fg, border) = (theme.panel, theme.text, theme.hairline);
    div()
        .id(("open-card", id))
        .absolute()
        .top_2()
        .right_2()
        .size(px(28.))
        .rounded_full()
        .bg(gpui::black().opacity(0.55))
        .hover(|s| s.bg(gpui::black().opacity(0.8)))
        .flex()
        .items_center()
        .justify_center()
        .invisible()
        .group_hover(GROUP, gpui::Styled::visible)
        .tooltip(move |window, cx| {
            Tooltip::new("Open link")
                .bg(bg)
                .text_color(fg)
                .border_color(border)
                .build(window, cx)
        })
        .child(icon("arrow-up-right", px(15.), gpui::white()))
}
