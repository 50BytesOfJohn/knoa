//! Catppuccin's four flavors, from catppuccin/palette. Mauve is the accent, as in most ports.

use super::{BRAND_RED, Theme, ai_soft, ai_text, below, card_hover_shadow, rgb};

pub(super) fn themes() -> Vec<Theme> {
    vec![
        theme("catppuccin-latte", "Catppuccin Latte", &LATTE),
        theme("catppuccin-frappe", "Catppuccin Frappé", &FRAPPE),
        theme("catppuccin-macchiato", "Catppuccin Macchiato", &MACCHIATO),
        theme("catppuccin-mocha", "Catppuccin Mocha", &MOCHA),
    ]
}

/// The part of a flavor Knoa uses, named as Catppuccin names it.
struct Flavor {
    dark: bool,
    red: u32,
    pink: u32,
    yellow: u32,
    green: u32,
    teal: u32,
    sky: u32,
    sapphire: u32,
    blue: u32,
    lavender: u32,
    mauve: u32,
    text: u32,
    subtext0: u32,
    overlay0: u32,
    surface1: u32,
    surface0: u32,
    base: u32,
    mantle: u32,
    crust: u32,
}

fn theme(id: &'static str, name: &'static str, f: &Flavor) -> Theme {
    let (canvas, panel, card, soft, hover) = if f.dark {
        (f.crust, f.mantle, f.base, f.base, f.surface0)
    } else {
        (f.crust, f.base, f.base, f.mantle, f.crust)
    };
    let hairline = if f.dark { f.surface0 } else { f.surface1 };
    // Code cards stay dark in Latte too, so they take Mocha's colors there.
    let (code_card, code_text) = if f.dark {
        (below(rgb(f.crust)), rgb(f.text))
    } else {
        (rgb(MOCHA.base), rgb(MOCHA.text))
    };
    // Latte's red is a crimson that reads as wine. The dark flavors' reds sit next to the
    // brand red, so they warn in pink.
    let danger = if f.dark { f.pink } else { f.red };
    Theme {
        id,
        name,
        dark: f.dark,
        canvas: rgb(canvas),
        panel: rgb(panel),
        hairline: rgb(hairline),
        card: rgb(card),
        card_hover_shadow: card_hover_shadow(f.dark),
        text: rgb(f.text),
        muted: rgb(f.subtext0),
        faint: rgb(f.overlay0),
        accent: rgb(f.mauve),
        on_accent: rgb(if f.dark { f.crust } else { f.base }),
        soft: rgb(soft),
        hover: rgb(hover),
        ai: rgb(BRAND_RED),
        ai_text: ai_text(f.dark),
        ai_soft: ai_soft(rgb(panel), f.dark),
        code_card,
        code_text,
        danger: rgb(danger),
        folders: [
            f.yellow, f.green, f.teal, f.sky, f.sapphire, f.blue, f.lavender, f.mauve,
        ]
        .map(rgb),
    }
}

const LATTE: Flavor = Flavor {
    dark: false,
    red: 0x00d2_0f39,
    pink: 0x00ea_76cb,
    yellow: 0x00df_8e1d,
    green: 0x0040_a02b,
    teal: 0x0017_9299,
    sky: 0x0004_a5e5,
    sapphire: 0x0020_9fb5,
    blue: 0x001e_66f5,
    lavender: 0x0072_87fd,
    mauve: 0x0088_39ef,
    text: 0x004c_4f69,
    subtext0: 0x006c_6f85,
    overlay0: 0x009c_a0b0,
    surface1: 0x00bc_c0cc,
    surface0: 0x00cc_d0da,
    base: 0x00ef_f1f5,
    mantle: 0x00e6_e9ef,
    crust: 0x00dc_e0e8,
};

const FRAPPE: Flavor = Flavor {
    dark: true,
    red: 0x00e7_8284,
    pink: 0x00f4_b8e4,
    yellow: 0x00e5_c890,
    green: 0x00a6_d189,
    teal: 0x0081_c8be,
    sky: 0x0099_d1db,
    sapphire: 0x0085_c1dc,
    blue: 0x008c_aaee,
    lavender: 0x00ba_bbf1,
    mauve: 0x00ca_9ee6,
    text: 0x00c6_d0f5,
    subtext0: 0x00a5_adce,
    overlay0: 0x0073_7994,
    surface1: 0x0051_576d,
    surface0: 0x0041_4559,
    base: 0x0030_3446,
    mantle: 0x0029_2c3c,
    crust: 0x0023_2634,
};

const MACCHIATO: Flavor = Flavor {
    dark: true,
    red: 0x00ed_8796,
    pink: 0x00f5_bde6,
    yellow: 0x00ee_d49f,
    green: 0x00a6_da95,
    teal: 0x008b_d5ca,
    sky: 0x0091_d7e3,
    sapphire: 0x007d_c4e4,
    blue: 0x008a_adf4,
    lavender: 0x00b7_bdf8,
    mauve: 0x00c6_a0f6,
    text: 0x00ca_d3f5,
    subtext0: 0x00a5_adcb,
    overlay0: 0x006e_738d,
    surface1: 0x0049_4d64,
    surface0: 0x0036_3a4f,
    base: 0x0024_273a,
    mantle: 0x001e_2030,
    crust: 0x0018_1926,
};

const MOCHA: Flavor = Flavor {
    dark: true,
    red: 0x00f3_8ba8,
    pink: 0x00f5_c2e7,
    yellow: 0x00f9_e2af,
    green: 0x00a6_e3a1,
    teal: 0x0094_e2d5,
    sky: 0x0089_dceb,
    sapphire: 0x0074_c7ec,
    blue: 0x0089_b4fa,
    lavender: 0x00b4_befe,
    mauve: 0x00cb_a6f7,
    text: 0x00cd_d6f4,
    subtext0: 0x00a6_adc8,
    overlay0: 0x006c_7086,
    surface1: 0x0045_475a,
    surface0: 0x0031_3244,
    base: 0x001e_1e2e,
    mantle: 0x0018_1825,
    crust: 0x0011_111b,
};
