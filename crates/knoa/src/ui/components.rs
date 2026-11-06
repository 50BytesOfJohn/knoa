//! Small building blocks shared by Knoa's views.

use gpui::{
    AnyElement, App, Div, ElementId, Hsla, InteractiveElement as _, IntoElement,
    ParentElement as _, Pixels, SharedString, StatefulInteractiveElement as _, Styled as _, Svg,
    div, px, svg,
};
use knoa_browser::{Browser, FULL_DISK_ACCESS_SETTINGS};
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

/// A problem to fix, in the danger colour and always with an icon, so it never passes for
/// the AI's red.
pub fn problem(text: impl Into<SharedString>, theme: &Theme) -> Div {
    div()
        .flex()
        .items_start()
        .gap_2()
        .text_sm()
        .text_color(theme.danger)
        .child(icon("circle-alert", px(14.), theme.danger).mt(px(3.)))
        .child(div().flex_1().min_w_0().child(text.into()))
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

/// Tells the user which installed browsers macOS hides from Knoa, with a button to the
/// setting that unblocks them. `None` when nothing is blocked.
pub fn blocked_browsers_notice(blocked: &[Browser], cx: &App) -> Option<Div> {
    if blocked.is_empty() {
        return None;
    }
    let theme = Theme::get(cx);
    let names: Vec<&str> = blocked.iter().map(|b| b.name()).collect();
    let text = format!(
        "{} is installed, but macOS keeps its bookmarks private. Give Knoa Full Disk Access, then restart Knoa.",
        names.join(" and ")
    );
    Some(
        div()
            .p_3()
            .flex()
            .flex_col()
            .items_start()
            .gap_2()
            .rounded(px(12.))
            .bg(theme.soft)
            .text_xs()
            .text_color(theme.muted)
            .child(text)
            .child(
                button("full-disk-access", "Open Privacy settings", Tone::Soft, cx)
                    .on_click(|_, _, cx| cx.open_url(FULL_DISK_ACCESS_SETTINGS)),
            ),
    )
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
