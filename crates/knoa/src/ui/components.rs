//! Small building blocks shared by Knoa's views.

use gpui::{
    AnyElement, AnyView, App, Div, ElementId, Hsla, InteractiveElement as _, IntoElement,
    ParentElement as _, Pixels, SharedString, StatefulInteractiveElement as _, Styled as _, Svg,
    Window, div, prelude::FluentBuilder as _, px, svg,
};
use gpui_component::tooltip::Tooltip;
use knoa_browser::{Browser, FULL_DISK_ACCESS_SETTINGS};
use knoa_core::{FolderIcon, Ink, Kind, protocol::max_chars::fits};

use crate::{
    assets::knoa_icon,
    ui::theme::{CARD_RADIUS, Theme},
};

/// How strongly the tinted finish fills a folder shape.
const FOLDER_TINT: f32 = 0.18;

/// A Lucide icon by file name, e.g. `"bookmark"`.
pub fn icon(name: &str, size: Pixels, color: Hsla) -> Svg {
    svg()
        .path(SharedString::from(format!("icons/{name}.svg")))
        .size(size)
        .flex_none()
        .text_color(color)
}

/// The same icon as [`icon`], for gpui-component pieces that take an `Icon`.
pub fn icon_asset(name: &str) -> gpui_component::Icon {
    gpui_component::Icon::empty().path(format!("icons/{name}.svg"))
}

/// How a folder's mark is drawn.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Finish {
    Line,
    /// The folder shape with a faint fill under its line, for the selected folder. Glyphs
    /// stay line only.
    Tinted,
}

/// A folder's mark: its glyph, or the folder shape when it has none.
pub fn folder_mark(glyph: Option<FolderIcon>, color: Hsla, size: Pixels, finish: Finish) -> Div {
    let layer = |path: SharedString, color: Hsla| {
        svg()
            .path(path)
            .absolute()
            .top_0()
            .left_0()
            .size(size)
            .text_color(color)
    };
    let tinted = finish == Finish::Tinted && glyph.is_none();
    div()
        .relative()
        .size(size)
        .flex_none()
        .when(tinted, |s| {
            s.child(layer(
                knoa_icon("folder-filled"),
                color.opacity(FOLDER_TINT),
            ))
        })
        .child(layer(folder_mark_path(glyph), color))
}

/// The line mark of [`folder_mark`], for gpui-component pieces that take an `Icon`.
pub fn folder_mark_asset(glyph: Option<FolderIcon>) -> gpui_component::Icon {
    gpui_component::Icon::empty().path(folder_mark_path(glyph))
}

fn folder_mark_path(glyph: Option<FolderIcon>) -> SharedString {
    knoa_icon(glyph.map_or("folder", FolderIcon::name))
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

/// `100000` as `100,000`.
pub fn grouped(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// Why `text` can't be saved when it's past the server's `max` characters for its field,
/// which would keep the change on this Mac only: "Instructions can be at most 10,000
/// characters."
pub fn too_long(what: &str, text: &str, max: usize) -> Option<String> {
    (!fits(text, max)).then(|| format!("{what} can be at most {} characters.", grouped(max as u64)))
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
    let ink = Ink::nth(usize::from(hash));
    div()
        .size(size)
        .flex_none()
        .rounded_full()
        .bg(theme.ink(ink).opacity(0.85))
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
            .rounded(CARD_RADIUS)
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
pub fn workspace_mark(name: &str, color: Ink, size: Pixels, cx: &App) -> Div {
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
        .bg(theme.ink(color))
        .flex()
        .items_center()
        .justify_center()
        .text_color(gpui::white())
        .text_size(size * 0.5)
        .font_weight(gpui::FontWeight::BOLD)
        .child(initial)
}

/// A tooltip in the theme's panel colors, with the key that does the same from the card
/// grid, if any.
pub fn tip(
    text: &'static str,
    action: Option<&'static dyn gpui::Action>,
    theme: &Theme,
) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
    let (bg, fg, border) = (theme.panel, theme.text, theme.hairline);
    move |window, cx| {
        let tooltip = Tooltip::new(text);
        let tooltip = match action {
            Some(action) => tooltip.action(action, Some("Cards")),
            None => tooltip,
        };
        tooltip
            .bg(bg)
            .text_color(fg)
            .border_color(border)
            .build(window, cx)
    }
}
