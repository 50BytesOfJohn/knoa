//! Knoa's own themes: Paper, the default, and its relatives.

use gpui::Hsla;
use knoa_core::Ink;

use super::{BRAND_RED, Theme, ai_text, card_hover_shadow, rgb};

pub(super) fn themes() -> Vec<Theme> {
    vec![paper(), sage(), night(), dusk()]
}

pub(super) fn paper() -> Theme {
    Theme {
        id: "paper",
        name: "Paper",
        dark: false,
        canvas: rgb(0x00ef_ece6),
        panel: rgb(0x00fb_faf7),
        hairline: rgb(0x00e2_ddd4),
        card: rgb(0x00ff_ffff),
        card_hover_shadow: card_hover_shadow(false),
        text: rgb(0x0022_201c),
        muted: rgb(0x006f_6a61),
        faint: rgb(0x00a3_9d92),
        accent: rgb(0x002f_5d50),
        on_accent: rgb(0x00ff_ffff),
        soft: rgb(0x00f0_ede6),
        hover: rgb(0x00ec_e8e0),
        ai: rgb(BRAND_RED),
        ai_text: ai_text(false),
        ai_soft: rgb(0x00fb_e7e0),
        code_card: rgb(0x0024_2a2e),
        code_text: rgb(0x00e8_ecee),
        // Wine, not red: errors must not look like the AI's mark.
        danger: rgb(0x008e_1538),
        folders: palette(false),
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
        card_hover_shadow: card_hover_shadow(true),
        text: rgb(0x00ec_eae6),
        muted: rgb(0x009c_9891),
        faint: rgb(0x0068_655f),
        accent: rgb(0x008f_c7b4),
        on_accent: rgb(0x0012_1314),
        soft: rgb(0x0026_2729),
        hover: rgb(0x002b_2d30),
        ai: rgb(BRAND_RED),
        ai_text: ai_text(true),
        ai_soft: rgb(0x003b_231d),
        code_card: rgb(0x000f_1215),
        code_text: rgb(0x00d8_dee2),
        danger: rgb(0x00f5_8fb8),
        folders: palette(true),
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

fn palette(dark: bool) -> [Hsla; Ink::COUNT] {
    let hex = if dark {
        [
            0x00d9_b04c,
            0x00a9_b45a,
            0x007f_bf6a,
            0x005f_a892,
            0x004f_bdbd,
            0x0074_b8e8,
            0x007c_9aec,
            0x00a8_8ae6,
            0x00c5_83c8,
            0x00e0_8abd,
            0x00b8_9078,
            0x0094_a1b2,
        ]
    } else {
        [
            0x00b8_891f,
            0x007f_8a2a,
            0x004f_8f3a,
            0x002f_6d5a,
            0x001f_8a8a,
            0x003a_8fc9,
            0x003f_62c4,
            0x007a_52c0,
            0x009a_4a9e,
            0x00c2_589a,
            0x008a_6248,
            0x005f_6b7a,
        ]
    };
    hex.map(rgb)
}
