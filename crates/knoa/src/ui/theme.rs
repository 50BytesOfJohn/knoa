//! Knoa's themes: colors, type, spacing and radii. Several themes, not only light and dark;
//! the default is a calm, solid light one. gpui-component's theme is derived from ours so
//! inputs and scrollbars match.

use gpui::{App, Global, Hsla, Pixels, hsla, px};

pub const DEFAULT_THEME: &str = "paper";

/// Space around and between floating panels.
pub const MARGIN: Pixels = px(12.);
pub const SIDEBAR_WIDTH: Pixels = px(232.);
pub const DETAIL_WIDTH: Pixels = px(340.);
pub const PANEL_RADIUS: Pixels = px(18.);
pub const CARD_RADIUS: Pixels = px(14.);
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
    pub folders: [Hsla; 8],
}

impl Global for Theme {}

impl Theme {
    pub fn get(cx: &App) -> &Theme {
        cx.global::<Theme>()
    }

    pub fn all() -> [Theme; 4] {
        [paper(), sage(), night(), dusk()]
    }

    pub fn by_id(id: &str) -> Theme {
        Theme::all()
            .into_iter()
            .find(|t| t.id == id)
            .unwrap_or_else(paper)
    }

    pub fn folder_color(&self, index: u8) -> Hsla {
        self.folders[usize::from(index) % self.folders.len()]
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

/// No reds or oranges: a folder dot must never pass for the AI's.
fn folder_palette(dark: bool) -> [Hsla; 8] {
    let lightness = if dark { 0.66 } else { 0.55 };
    [0.12, 0.2, 0.36, 0.47, 0.57, 0.65, 0.74, 0.84].map(|hue| hsla(hue, 0.55, lightness, 1.))
}

fn paper() -> Theme {
    Theme {
        id: "paper",
        name: "Paper",
        dark: false,
        canvas: rgb(0x00ef_ece6),
        panel: rgb(0x00fb_faf7),
        hairline: rgb(0x00e2_ddd4),
        card: rgb(0x00ff_ffff),
        card_hover_shadow: hsla(0.08, 0.3, 0.2, 0.14),
        text: rgb(0x0022_201c),
        muted: rgb(0x006f_6a61),
        faint: rgb(0x00a3_9d92),
        accent: rgb(0x002f_5d50),
        on_accent: rgb(0x00ff_ffff),
        soft: rgb(0x00f0_ede6),
        hover: rgb(0x00ec_e8e0),
        ai: rgb(BRAND_RED),
        ai_text: rgb(0x00b8_402a),
        ai_soft: rgb(0x00fb_e7e0),
        code_card: rgb(0x0024_2a2e),
        code_text: rgb(0x00e8_ecee),
        // Wine, not red: errors must not look like the AI's mark.
        danger: rgb(0x008e_1538),
        folders: folder_palette(false),
    }
}

fn sage() -> Theme {
    Theme {
        id: "sage",
        name: "Sage",
        canvas: rgb(0x00e3_e9e1),
        panel: rgb(0x00f6_f8f4),
        hairline: rgb(0x00d5_ddd2),
        soft: rgb(0x00e9_eee6),
        hover: rgb(0x00e2_e8de),
        accent: rgb(0x003d_6b45),
        ..paper()
    }
}

fn night() -> Theme {
    Theme {
        id: "night",
        name: "Night",
        dark: true,
        canvas: rgb(0x0012_1314),
        panel: rgb(0x001c_1d1f),
        hairline: rgb(0x002e_3033),
        card: rgb(0x0023_2427),
        card_hover_shadow: hsla(0., 0., 0., 0.45),
        text: rgb(0x00ec_eae6),
        muted: rgb(0x009c_9891),
        faint: rgb(0x0068_655f),
        accent: rgb(0x008f_c7b4),
        on_accent: rgb(0x0012_1314),
        soft: rgb(0x0026_2729),
        hover: rgb(0x002b_2d30),
        ai: rgb(BRAND_RED),
        ai_text: rgb(0x00f0_8a70),
        ai_soft: rgb(0x003b_231d),
        code_card: rgb(0x000f_1215),
        code_text: rgb(0x00d8_dee2),
        danger: rgb(0x00f5_8fb8),
        folders: folder_palette(true),
    }
}

fn dusk() -> Theme {
    Theme {
        id: "dusk",
        name: "Dusk",
        canvas: rgb(0x0017_1520),
        panel: rgb(0x0021_1e2c),
        hairline: rgb(0x0034_3045),
        card: rgb(0x0028_2536),
        soft: rgb(0x002c_2839),
        hover: rgb(0x0032_2e42),
        accent: rgb(0x00c3_a6f2),
        ..night()
    }
}
