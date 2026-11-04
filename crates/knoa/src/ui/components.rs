//! Small building blocks shared by Knoa's views.

use gpui::{
    AnyElement, App, Div, ElementId, Hsla, InteractiveElement as _, IntoElement,
    ParentElement as _, Pixels, SharedString, Styled as _, Svg, div, px, svg,
};
use knoa_core::Kind;

use crate::ui::theme::Theme;

/// A Lucide icon by file name, e.g. `"bookmark"`.
pub fn icon(name: &str, size: Pixels, color: Hsla) -> Svg {
    svg()
        .path(SharedString::from(format!("icons/{name}.svg")))
        .size(size)
        .flex_none()
        .text_color(color)
}

pub fn kind_icon(kind: Option<Kind>) -> &'static str {
    match kind {
        Some(Kind::Article) => "file-text",
        Some(Kind::Video) => "play",
        Some(Kind::Movie) => "clapperboard",
        Some(Kind::Recipe) => "chef-hat",
        Some(Kind::Product) => "shopping-bag",
        Some(Kind::Repository) => "code",
        Some(Kind::Book) => "book-open",
        Some(Kind::Image) => "image",
        Some(Kind::App) => "app-window",
        Some(Kind::Podcast) => "mic",
        Some(Kind::Place) => "map-pin",
        Some(Kind::Other) | None => "link",
    }
}

/// How a button looks.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// Filled with the accent.
    Primary,
    /// A soft fill.
    Soft,
    /// Text only, with a hover fill.
    Ghost,
}

/// A rounded button; attach `on_click` to it.
pub fn button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    tone: Tone,
    cx: &App,
) -> gpui::Stateful<Div> {
    let theme = Theme::get(cx);
    let (bg, fg, hover) = match tone {
        Tone::Primary => (theme.accent, theme.on_accent, theme.accent.opacity(0.88)),
        Tone::Soft => (theme.soft, theme.text, theme.hover),
        Tone::Ghost => (gpui::transparent_black(), theme.muted, theme.soft),
    };
    div()
        .id(id.into())
        .flex()
        .items_center()
        .justify_center()
        .gap_1p5()
        .h(px(32.))
        .px_3p5()
        .rounded_full()
        .bg(bg)
        .text_color(fg)
        .text_sm()
        .font_weight(gpui::FontWeight::MEDIUM)
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .child(label.into())
}

/// A small rounded label.
pub fn chip(label: impl Into<SharedString>, bg: Hsla, fg: Hsla) -> Div {
    div()
        .flex()
        .items_center()
        .gap_1()
        .h(px(22.))
        .px_2()
        .rounded_full()
        .bg(bg)
        .text_color(fg)
        .text_xs()
        .child(label.into())
}

/// The site's first letter on a colored disc: a stand-in for favicons, which would mean
/// asking a third party about every bookmarked site.
pub fn site_mark(site: &str, size: Pixels, cx: &App) -> AnyElement {
    let theme = Theme::get(cx);
    let letter = site
        .chars()
        .find(char::is_ascii_alphanumeric)
        .unwrap_or('•')
        .to_ascii_uppercase();
    let hash = site.bytes().fold(0u8, u8::wrapping_add);
    div()
        .size(size)
        .flex_none()
        .rounded_full()
        .bg(theme.folder_color(hash).opacity(0.85))
        .flex()
        .items_center()
        .justify_center()
        .text_color(gpui::white())
        .text_size(size * 0.55)
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .child(letter.to_string())
        .into_any_element()
}

/// The coloured square with a workspace's initial.
pub fn workspace_mark(name: &str, color: u8, size: Pixels, cx: &App) -> Div {
    let theme = Theme::get(cx);
    let initial = name
        .chars()
        .next()
        .unwrap_or('K')
        .to_uppercase()
        .to_string();
    div()
        .size(size)
        .flex_none()
        .rounded(size * 0.28)
        .bg(theme.folder_color(color))
        .flex()
        .items_center()
        .justify_center()
        .text_color(gpui::white())
        .text_size(size * 0.5)
        .font_weight(gpui::FontWeight::BOLD)
        .child(initial)
}
