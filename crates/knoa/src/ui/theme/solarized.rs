//! Solarized, light and dark, from altercation/solarized. Both share one set of accents
//! and swap the base tones, as Solarized intends. Magenta marks danger and is the Rose
//! ink.

use gpui::Hsla;

use super::{
    BRAND_RED, Theme, ai_soft, ai_text, below, card_hover_shadow, cocoa, inks, mix, pine, rgb,
    slate, spread,
};

pub(super) fn themes() -> Vec<Theme> {
    vec![light(), dark()]
}

const BASE03: u32 = 0x0000_2b36;
const BASE02: u32 = 0x0007_3642;
const BASE01: u32 = 0x0058_6e75;
const BASE00: u32 = 0x0065_7b83;
const BASE0: u32 = 0x0083_9496;
const BASE1: u32 = 0x0093_a1a1;
const BASE2: u32 = 0x00ee_e8d5;
const BASE3: u32 = 0x00fd_f6e3;
const BLUE: u32 = 0x0026_8bd2;
const MAGENTA: u32 = 0x00d3_3682;

// Solarized has two background tones per mode, so lines and hovers are mixed from a
// background and the next content tone.

fn dark() -> Theme {
    let panel = rgb(BASE02);
    Theme {
        id: "solarized-dark",
        name: "Solarized Dark",
        dark: true,
        canvas: rgb(BASE03),
        panel,
        hairline: mix(panel, rgb(BASE01), 0.35),
        card: panel,
        card_hover_shadow: card_hover_shadow(true),
        text: rgb(BASE1),
        muted: rgb(BASE0),
        faint: rgb(BASE01),
        accent: rgb(BLUE),
        on_accent: rgb(BASE3),
        soft: mix(panel, rgb(BASE01), 0.2),
        hover: mix(panel, rgb(BASE01), 0.25),
        ai: rgb(BRAND_RED),
        ai_text: ai_text(true),
        ai_soft: ai_soft(panel, true),
        code_card: below(rgb(BASE03)),
        code_text: rgb(BASE1),
        danger: rgb(MAGENTA),
        folders: folders(true),
    }
}

fn light() -> Theme {
    let panel = rgb(BASE3);
    Theme {
        id: "solarized-light",
        name: "Solarized Light",
        dark: false,
        canvas: rgb(BASE2),
        panel,
        hairline: mix(rgb(BASE2), rgb(BASE1), 0.35),
        card: panel,
        card_hover_shadow: card_hover_shadow(false),
        text: rgb(BASE01),
        muted: rgb(BASE00),
        faint: rgb(BASE1),
        accent: rgb(BLUE),
        on_accent: rgb(BASE3),
        soft: rgb(BASE2),
        hover: mix(rgb(BASE2), rgb(BASE1), 0.2),
        ai: rgb(BRAND_RED),
        ai_text: ai_text(false),
        ai_soft: ai_soft(panel, false),
        code_card: rgb(BASE03),
        code_text: rgb(BASE1),
        danger: rgb(MAGENTA),
        folders: folders(false),
    }
}

/// Yellow, green, cyan, blue and violet, with the steps between them. The accents are
/// shared, so only the quiet inks differ between modes.
fn folders(dark: bool) -> [Hsla; 12] {
    let hues = [0x00b5_8900, 0x0085_9900, 0x002a_a198, BLUE, 0x006c_71c4].map(rgb);
    let [yellow, green, cyan, blue, _] = hues;
    inks(
        spread(hues),
        pine(green, cyan, dark),
        rgb(MAGENTA),
        cocoa(yellow, dark),
        slate(blue, dark),
    )
}
