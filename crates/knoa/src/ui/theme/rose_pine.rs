//! Rosé Pine, Moon and Dawn, from rose-pine/neovim's palette.

use super::{BRAND_RED, Theme, ai_soft, ai_text, between, card_hover_shadow, rgb};

pub(super) fn themes() -> Vec<Theme> {
    vec![
        theme("rose-pine-dawn", "Rosé Pine Dawn", &DAWN),
        theme("rose-pine", "Rosé Pine", &MAIN),
        theme("rose-pine-moon", "Rosé Pine Moon", &MOON),
    ]
}

/// A variant, named as Rosé Pine names its colors. `nc` is the unfocused-window base.
struct Variant {
    dark: bool,
    nc: u32,
    base: u32,
    surface: u32,
    overlay: u32,
    muted: u32,
    subtle: u32,
    text: u32,
    love: u32,
    gold: u32,
    pine: u32,
    foam: u32,
    iris: u32,
    leaf: u32,
    highlight_low: u32,
    highlight_med: u32,
}

fn theme(id: &'static str, name: &'static str, v: &Variant) -> Theme {
    // Dark variants' lowest highlight is too close to their surface to show as a hover.
    let (soft, hover) = if v.dark {
        (v.overlay, v.overlay)
    } else {
        (v.highlight_low, v.overlay)
    };
    // Code cards stay dark in Dawn too, so they take the main variant's colors there.
    let (code_card, code_text) = if v.dark {
        (v.nc, v.text)
    } else {
        (MAIN.base, MAIN.text)
    };
    let [gold, leaf, foam, pine, iris, love] =
        [v.gold, v.leaf, v.foam, v.pine, v.iris, v.love].map(rgb);
    Theme {
        id,
        name,
        dark: v.dark,
        canvas: rgb(v.base),
        panel: rgb(v.surface),
        hairline: rgb(v.highlight_med),
        card: rgb(v.surface),
        card_hover_shadow: card_hover_shadow(v.dark),
        text: rgb(v.text),
        muted: rgb(v.subtle),
        faint: rgb(v.muted),
        accent: rgb(v.iris),
        on_accent: rgb(if v.dark { v.base } else { v.surface }),
        soft: rgb(soft),
        hover: rgb(hover),
        ai: rgb(BRAND_RED),
        ai_text: ai_text(v.dark),
        ai_soft: ai_soft(rgb(v.surface), v.dark),
        code_card: rgb(code_card),
        code_text: rgb(code_text),
        // Love is Rosé Pine's own error color, a pink well apart from the brand red.
        danger: love,
        // Six hues without love, so the gaps get the midpoints between neighbors.
        folders: [
            gold,
            between(gold, leaf),
            leaf,
            foam,
            pine,
            between(pine, iris),
            iris,
            between(iris, love),
        ],
    }
}

const MAIN: Variant = Variant {
    dark: true,
    nc: 0x0016_141f,
    base: 0x0019_1724,
    surface: 0x001f_1d2e,
    overlay: 0x0026_233a,
    muted: 0x006e_6a86,
    subtle: 0x0090_8caa,
    text: 0x00e0_def4,
    love: 0x00eb_6f92,
    gold: 0x00f6_c177,
    pine: 0x0031_748f,
    foam: 0x009c_cfd8,
    iris: 0x00c4_a7e7,
    leaf: 0x0095_b1ac,
    highlight_low: 0x0021_202e,
    highlight_med: 0x0040_3d52,
};

const MOON: Variant = Variant {
    dark: true,
    nc: 0x001f_1d30,
    base: 0x0023_2136,
    surface: 0x002a_273f,
    overlay: 0x0039_3552,
    muted: 0x006e_6a86,
    subtle: 0x0090_8caa,
    text: 0x00e0_def4,
    love: 0x00eb_6f92,
    gold: 0x00f6_c177,
    pine: 0x003e_8fb0,
    foam: 0x009c_cfd8,
    iris: 0x00c4_a7e7,
    leaf: 0x0095_b1ac,
    highlight_low: 0x002a_283e,
    highlight_med: 0x0044_415a,
};

const DAWN: Variant = Variant {
    dark: false,
    nc: 0x00f8_f0e7,
    base: 0x00fa_f4ed,
    surface: 0x00ff_faf3,
    overlay: 0x00f2_e9e1,
    muted: 0x0098_93a5,
    subtle: 0x0079_7593,
    text: 0x0046_4261,
    love: 0x00b4_637a,
    gold: 0x00ea_9d34,
    pine: 0x0028_6983,
    foam: 0x0056_949f,
    iris: 0x0090_7aa9,
    leaf: 0x006d_8f89,
    highlight_low: 0x00f4_ede8,
    highlight_med: 0x00df_dad9,
};
