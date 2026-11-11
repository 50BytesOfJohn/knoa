//! Knoa's themes: colors, type, spacing and radii. Several themes, not only light and dark;
//! the default is a calm, solid light one. gpui-component's theme is derived from ours so
//! inputs and scrollbars match.
//!
//! Besides Knoa's own themes there are ports of popular editor themes, one module per
//! family, built from each family's published palette. Red means only the brand and the
//! AI in every theme, so a port takes its danger and folder colors from the family's
//! other hues. Where a family has no quiet green, brown or grey-blue for the Pine, Cocoa
//! and Slate inks, they are its own green, yellow and blue dulled to the shade of Knoa's.

mod catppuccin;
mod dracula;
mod everforest;
mod gruvbox;
mod kanagawa;
mod knoa;
mod nord;
mod one;
mod rose_pine;
mod solarized;
mod tokyo_night;

use gpui::{App, Global, Hsla, Pixels, hsla, px};
use knoa_core::{FolderColor, Hue, Ink};

pub const DEFAULT_THEME: &str = "paper";

/// Space between the window's edge and the panels that float against it. The largest
/// space in the layout, so the panels read as floating and nothing inside them is
/// spaced wider than the room around them.
pub const MARGIN: Pixels = px(12.);
/// Space between a panel and the content beside it, the same as between cards.
pub const GUTTER: Pixels = px(16.);
pub const SIDEBAR_WIDTH: Pixels = px(240.);
pub const DETAIL_WIDTH: Pixels = px(340.);
/// The same curve macOS 26 gives a window without a toolbar, so the panels echo the
/// window's corners. Every floating panel shares it.
pub const PANEL_RADIUS: Pixels = px(16.);
pub const CARD_RADIUS: Pixels = px(10.);
/// Rows, buttons and items inside a panel. Rows sit 8 px in from a panel's edge, so
/// this is concentric with the panel's corners.
pub const ROW_RADIUS: Pixels = px(8.);
/// A place in the sidebar, or a page in the settings sidebar that mirrors it.
pub const ROW_HEIGHT: Pixels = px(32.);
pub const SERIF: &str = "Charter";
/// The logo's full stop. The AI mascot is that dot come alive, so AI-made values share it.
pub const BRAND_RED: u32 = 0x00e0_573c;

#[derive(Debug, Clone)]
pub struct Theme {
    pub id: &'static str,
    pub name: &'static str,
    pub dark: bool,
    /// The window background behind the floating panels.
    pub canvas: Hsla,
    pub panel: Hsla,
    pub hairline: Hsla,
    pub card: Hsla,
    pub card_hover_shadow: Hsla,
    pub text: Hsla,
    pub muted: Hsla,
    pub faint: Hsla,
    /// The theme's accent: selected place, primary buttons.
    pub accent: Hsla,
    pub on_accent: Hsla,
    pub soft: Hsla,
    pub hover: Hsla,
    /// AI marks and the mascot: the brand red, separate from the accent on purpose.
    pub ai: Hsla,
    /// AI-made text, a shade of the brand red that stays readable at small sizes.
    pub ai_text: Hsla,
    pub ai_soft: Hsla,
    pub code_card: Hsla,
    pub code_text: Hsla,
    pub danger: Hsla,
    /// The inks, in [`Ink`] order, for folders, workspaces and site marks. No reds or
    /// oranges: a folder must never pass for the AI's mark. A free hue takes its shade
    /// from the inks nearest it, and [`Hue`] keeps it out of that range too.
    pub folders: [Hsla; Ink::COUNT],
}

impl Global for Theme {}

impl Theme {
    pub fn get(cx: &App) -> &Theme {
        cx.global::<Theme>()
    }

    /// Knoa's own themes first, then the ported families in alphabetical order.
    pub fn all() -> Vec<Theme> {
        [
            knoa::themes(),
            catppuccin::themes(),
            dracula::themes(),
            everforest::themes(),
            gruvbox::themes(),
            kanagawa::themes(),
            nord::themes(),
            one::themes(),
            rose_pine::themes(),
            solarized::themes(),
            tokyo_night::themes(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    pub fn by_id(id: &str) -> Theme {
        Theme::all()
            .into_iter()
            .find(|t| t.id == id)
            .unwrap_or_else(knoa::paper)
    }

    pub fn ink(&self, ink: Ink) -> Hsla {
        self.folders[ink.index()]
    }

    pub fn folder_color(&self, color: FolderColor) -> Hsla {
        match color {
            FolderColor::Ink(ink) => self.ink(ink),
            FolderColor::Hue(hue) => self.hue(hue),
        }
    }

    /// A hue in the shade of the vivid inks on either side of it, blended by how close
    /// it sits to each, so a custom color matches the swatches beside it. Inks are picked
    /// with a different lightness per hue, since equal HSL lightness doesn't look equal
    /// across hues, so no single shade could match them all. Past the first or last ink
    /// it takes that ink's shade.
    pub fn hue(&self, hue: Hue) -> Hsla {
        let hue = f32::from(hue.degrees()) / 360.;
        let mut anchors: Vec<Hsla> = Ink::ALL
            .into_iter()
            .filter(|ink| !ink.is_quiet())
            .map(|ink| self.ink(ink))
            .collect();
        // Ported themes' inks are mixes, not always in hue order.
        anchors.sort_by(|a, b| a.h.total_cmp(&b.h));
        let (below, above) = match anchors.iter().position(|ink| ink.h >= hue) {
            Some(0) => (anchors[0], anchors[0]),
            Some(next) => (anchors[next - 1], anchors[next]),
            None => (anchors[anchors.len() - 1], anchors[anchors.len() - 1]),
        };
        // `position` found the first ink at or past `hue`, so the span is never zero
        // unless both are the same ink.
        let span = above.h - below.h;
        let t = if span > 0. {
            (hue - below.h) / span
        } else {
            0.
        };
        let blend = |from: f32, to: f32| from + (to - from) * t;
        hsla(hue, blend(below.s, above.s), blend(below.l, above.l), 1.)
    }

    /// Makes `id` the current theme, here and in gpui-component.
    pub fn apply(id: &str, cx: &mut App) {
        let theme = Theme::by_id(id);
        let mode = if theme.dark {
            gpui_component::ThemeMode::Dark
        } else {
            gpui_component::ThemeMode::Light
        };
        // `update`, not `global_mut`: it carries color edits into the semantic tokens that
        // switches and radios read. The mode goes first, in its own edit, because a mode
        // change reloads gpui-component's built-in theme over any colors set alongside it.
        gpui_component::Theme::update(cx, |component| component.mode = mode);
        gpui_component::Theme::update(cx, |component| theme.style_component(component));
        cx.set_global(theme);
        cx.refresh_windows();
    }

    fn style_component(&self, component: &mut gpui_component::Theme) {
        let c = &mut component.colors;
        c.background = self.panel;
        c.foreground = self.text;
        c.border = self.hairline;
        c.input = self.hairline;
        c.ring = self.accent.opacity(0.5);
        c.caret = self.accent;
        c.selection = self.accent.opacity(0.25);
        c.muted = self.soft;
        c.muted_foreground = self.muted;
        c.primary = self.accent;
        // A white thumb on every track, as macOS draws its switches.
        c.switch = self.faint.opacity(0.55);
        c.switch_thumb = gpui::white();
        c.popover = self.panel;
        c.popover_foreground = self.text;
        c.scrollbar = gpui::transparent_black();
        c.scrollbar_thumb = self.faint.opacity(0.5);
        component.font_size = px(14.);
        component.radius = px(10.);
    }
}

fn rgb(hex: u32) -> Hsla {
    gpui::rgb(hex).into()
}

/// A color given the way some palettes publish theirs: degrees and percentages.
fn hsl(hue: f32, saturation: f32, lightness: f32) -> Hsla {
    hsla(hue / 360., saturation / 100., lightness / 100., 1.)
}

/// `over` laid on `base` at `amount` opacity. For the steps a palette doesn't publish.
fn mix(base: Hsla, over: Hsla, amount: f32) -> Hsla {
    base.blend(over.opacity(amount))
}

/// Halfway between two palette colors, for families short of folder hues.
fn between(a: Hsla, b: Hsla) -> Hsla {
    mix(a, b, 0.5)
}

/// Eight folder colors from five hues: each hue and the midpoint to the next, ending
/// halfway to the last. For families with too few hues once red and orange are out. They
/// fill every ink but Pine, Rose, Cocoa and Slate.
fn spread(hues: [Hsla; 5]) -> [Hsla; 8] {
    std::array::from_fn(|slot| {
        let hue = hues[slot / 2];
        if slot % 2 == 0 {
            hue
        } else {
            between(hue, hues[slot / 2 + 1])
        }
    })
}

/// The inks from a family's eight folder colors, which run Ochre to Plum without Pine,
/// and the four it picks for the rest.
fn inks(eight: [Hsla; 8], pine: Hsla, rose: Hsla, cocoa: Hsla, slate: Hsla) -> [Hsla; Ink::COUNT] {
    let [ochre, olive, moss, teal, sky, cobalt, violet, plum] = eight;
    [
        ochre, olive, moss, pine, teal, sky, cobalt, violet, plum, rose, cocoa, slate,
    ]
}

/// Pine from a family's green and teal, for families with no quiet green. Mixing only
/// moves between a palette's colors and none of them is this muted, so it takes their hue
/// and Knoa's own Pine's saturation and lightness.
fn pine(green: Hsla, teal: Hsla, dark: bool) -> Hsla {
    let (saturation, lightness) = if dark { (0.3, 0.52) } else { (0.4, 0.31) };
    hsla(between(green, teal).h, saturation, lightness, 1.)
}

/// Cocoa from a family's yellow, like [`pine`]. Turned a little toward red, since a
/// family's orange, the natural brown, is kept for the AI.
fn cocoa(yellow: Hsla, dark: bool) -> Hsla {
    let (saturation, lightness) = if dark { (0.3, 0.6) } else { (0.31, 0.41) };
    hsla(
        (yellow.h - 12. / 360.).rem_euclid(1.),
        saturation,
        lightness,
        1.,
    )
}

/// Slate from a family's blue, like [`pine`].
fn slate(blue: Hsla, dark: bool) -> Hsla {
    let (saturation, lightness) = if dark { (0.15, 0.64) } else { (0.12, 0.43) };
    hsla(blue.h, saturation, lightness, 1.)
}

/// The brand red's text shade, the same in every theme of a mode.
fn ai_text(dark: bool) -> Hsla {
    rgb(if dark { 0x00f0_8a70 } else { 0x00b8_402a })
}

/// The brand red laid on `panel`, behind AI-made values.
fn ai_soft(panel: Hsla, dark: bool) -> Hsla {
    mix(panel, rgb(BRAND_RED), if dark { 0.16 } else { 0.12 })
}

/// A step below `canvas`, for code cards in palettes whose darkest color is the canvas.
fn below(canvas: Hsla) -> Hsla {
    mix(canvas, gpui::black(), 0.3)
}

fn card_hover_shadow(dark: bool) -> Hsla {
    if dark {
        hsla(0., 0., 0., 0.45)
    } else {
        hsla(0.08, 0.3, 0.2, 0.14)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use knoa_core::{Hue, Ink};

    use super::Theme;

    // Settings keep the theme by id, so a repeated id would quietly open the wrong theme.
    #[test]
    fn theme_ids_are_unique() {
        let themes = Theme::all();
        let ids: HashSet<_> = themes.iter().map(|t| t.id).collect();
        assert_eq!(ids.len(), themes.len());
    }

    #[test]
    fn a_custom_hue_at_an_inks_hue_takes_that_inks_shade() {
        let paper = Theme::by_id("paper");
        let cobalt = paper.ink(Ink::Cobalt);
        let custom = paper.hue(Hue::clamped(cobalt.h * 360.));
        assert!(
            (custom.s - cobalt.s).abs() < 0.01,
            "{custom:?} vs {cobalt:?}"
        );
        assert!(
            (custom.l - cobalt.l).abs() < 0.01,
            "{custom:?} vs {cobalt:?}"
        );
    }
}
