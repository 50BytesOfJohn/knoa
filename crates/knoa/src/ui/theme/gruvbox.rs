//! Gruvbox, dark and light, from morhetz/gruvbox. Aqua is the accent, purple marks
//! danger, and folders take two shades each of yellow, green, aqua and blue.

use super::{BRAND_RED, Theme, ai_soft, ai_text, card_hover_shadow, rgb};

pub(super) fn themes() -> Vec<Theme> {
    vec![light(), dark()]
}

fn dark() -> Theme {
    let panel = rgb(0x0032_302f);
    Theme {
        id: "gruvbox-dark",
        name: "Gruvbox Dark",
        dark: true,
        canvas: rgb(0x0028_2828),
        panel,
        hairline: rgb(0x0050_4945),
        card: rgb(0x003c_3836),
        card_hover_shadow: card_hover_shadow(true),
        text: rgb(0x00eb_dbb2),
        muted: rgb(0x00a8_9984),
        faint: rgb(0x007c_6f64),
        accent: rgb(0x008e_c07c),
        on_accent: rgb(0x0028_2828),
        soft: rgb(0x003c_3836),
        hover: rgb(0x003c_3836),
        ai: rgb(BRAND_RED),
        ai_text: ai_text(true),
        ai_soft: ai_soft(panel, true),
        code_card: rgb(0x001d_2021),
        code_text: rgb(0x00eb_dbb2),
        danger: rgb(0x00d3_869b),
        folders: [
            0x00fa_bd2f,
            0x00d7_9921,
            0x00b8_bb26,
            0x0098_971a,
            0x008e_c07c,
            0x0068_9d6a,
            0x0083_a598,
            0x0045_8588,
        ]
        .map(rgb),
    }
}

fn light() -> Theme {
    let panel = rgb(0x00fb_f1c7);
    Theme {
        id: "gruvbox-light",
        name: "Gruvbox Light",
        dark: false,
        canvas: rgb(0x00f2_e5bc),
        panel,
        hairline: rgb(0x00d5_c4a1),
        card: rgb(0x00f9_f5d7),
        card_hover_shadow: card_hover_shadow(false),
        text: rgb(0x003c_3836),
        muted: rgb(0x007c_6f64),
        faint: rgb(0x0092_8374),
        accent: rgb(0x0042_7b58),
        on_accent: panel,
        soft: rgb(0x00eb_dbb2),
        hover: rgb(0x00eb_dbb2),
        ai: rgb(BRAND_RED),
        ai_text: ai_text(false),
        ai_soft: ai_soft(panel, false),
        code_card: rgb(0x0028_2828),
        code_text: rgb(0x00eb_dbb2),
        danger: rgb(0x008f_3f71),
        folders: [
            0x00d7_9921,
            0x00b5_7614,
            0x0098_971a,
            0x0079_740e,
            0x0068_9d6a,
            0x0042_7b58,
            0x0045_8588,
            0x0007_6678,
        ]
        .map(rgb),
    }
}
