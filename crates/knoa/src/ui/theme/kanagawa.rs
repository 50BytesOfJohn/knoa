//! Kanagawa Wave and Lotus, from rebelot/kanagawa.nvim's palette. Danger takes the
//! sakura and lotus pinks, which sit well apart from the brand red.

use super::{BRAND_RED, Theme, ai_soft, ai_text, card_hover_shadow, rgb};

pub(super) fn themes() -> Vec<Theme> {
    vec![lotus(), wave()]
}

const SUMI_INK_3: u32 = 0x001f_1f28;
const FUJI_WHITE: u32 = 0x00dc_d7ba;

fn wave() -> Theme {
    let panel = rgb(SUMI_INK_3);
    Theme {
        id: "kanagawa-wave",
        name: "Kanagawa Wave",
        dark: true,
        canvas: rgb(0x0018_1820),
        panel,
        hairline: rgb(0x0036_3646),
        card: rgb(0x002a_2a37),
        card_hover_shadow: card_hover_shadow(true),
        text: rgb(FUJI_WHITE),
        muted: rgb(0x00c8_c093),
        faint: rgb(0x0072_7169),
        accent: rgb(0x007e_9cd8),
        on_accent: rgb(0x0016_161d),
        soft: rgb(0x002a_2a37),
        hover: rgb(0x0036_3646),
        ai: rgb(BRAND_RED),
        ai_text: ai_text(true),
        ai_soft: ai_soft(panel, true),
        code_card: rgb(0x0016_161d),
        code_text: rgb(FUJI_WHITE),
        danger: rgb(0x00d2_7e99),
        folders: [
            0x00e6_c384,
            0x0098_bb6c,
            0x0076_946a,
            0x007a_a89f,
            0x007f_b4ca,
            0x007e_9cd8,
            0x0095_7fb8,
            0x0093_8aa9,
        ]
        .map(rgb),
    }
}

fn lotus() -> Theme {
    let panel = rgb(0x00f2_ecbc);
    Theme {
        id: "kanagawa-lotus",
        name: "Kanagawa Lotus",
        dark: false,
        canvas: rgb(0x00e5_ddb0),
        panel,
        hairline: rgb(0x00d5_cea3),
        card: panel,
        card_hover_shadow: card_hover_shadow(false),
        text: rgb(0x0054_5464),
        muted: rgb(0x0071_6e61),
        faint: rgb(0x008a_8980),
        accent: rgb(0x004d_699b),
        on_accent: panel,
        soft: rgb(0x00e7_dba0),
        hover: rgb(0x00e5_ddb0),
        ai: rgb(BRAND_RED),
        ai_text: ai_text(false),
        ai_soft: ai_soft(panel, false),
        code_card: rgb(SUMI_INK_3),
        code_text: rgb(FUJI_WHITE),
        danger: rgb(0x00b3_5b79),
        folders: [
            0x00de_9800,
            0x006f_894e,
            0x006e_915f,
            0x0059_7b75,
            0x0066_93bf,
            0x004d_699b,
            0x0062_4c83,
            0x005d_57a3,
        ]
        .map(rgb),
    }
}
