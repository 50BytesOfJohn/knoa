//! Nord, from nordtheme/nord: Polar Night for surfaces, Snow Storm for text, Frost for
//! the accent and Aurora for the rest. Aurora's red and orange are left out, so its
//! purple marks danger.

use super::{BRAND_RED, Theme, ai_soft, ai_text, below, between, card_hover_shadow, mix, rgb};

pub(super) fn themes() -> Vec<Theme> {
    vec![nord()]
}

fn nord() -> Theme {
    let [nord0, nord1, nord2, nord3] =
        [0x002e_3440, 0x003b_4252, 0x0043_4c5e, 0x004c_566a].map(rgb);
    let [nord4, nord6] = [0x00d8_dee9, 0x00ec_eff4].map(rgb);
    let [nord7, nord8, nord9, nord10] =
        [0x008f_bcbb, 0x0088_c0d0, 0x0081_a1c1, 0x005e_81ac].map(rgb);
    let [nord13, nord14, nord15] = [0x00eb_cb8b, 0x00a3_be8c, 0x00b4_8ead].map(rgb);
    Theme {
        id: "nord",
        name: "Nord",
        dark: true,
        canvas: nord0,
        panel: nord1,
        hairline: nord2,
        card: nord1,
        card_hover_shadow: card_hover_shadow(true),
        text: nord6,
        muted: nord4,
        // nord3 is meant for comments on nord0 and nearly vanishes on nord1.
        faint: mix(nord3, nord4, 0.35),
        accent: nord8,
        on_accent: nord0,
        soft: nord2,
        hover: nord3,
        ai: rgb(BRAND_RED),
        ai_text: ai_text(true),
        ai_soft: ai_soft(nord1, true),
        code_card: below(nord0),
        code_text: nord4,
        danger: nord15,
        folders: [
            nord13,
            between(nord13, nord14),
            nord14,
            nord7,
            nord8,
            nord9,
            nord10,
            between(nord10, nord15),
        ],
    }
}
