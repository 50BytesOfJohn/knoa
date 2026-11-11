//! Dracula and its light sibling Alucard, from the spec at draculatheme.com/spec,
//! including its UI background steps. Both reds are true reds, so danger takes the pink,
//! which is also the Rose ink.

use super::{
    BRAND_RED, Theme, ai_soft, ai_text, between, card_hover_shadow, cocoa, inks, mix, pine, rgb,
    slate,
};

pub(super) fn themes() -> Vec<Theme> {
    vec![alucard(), dracula()]
}

fn dracula() -> Theme {
    let background = rgb(0x0028_2a36);
    let foreground = rgb(0x00f8_f8f2);
    let comment = rgb(0x0062_72a4);
    Theme {
        id: "dracula",
        name: "Dracula",
        dark: true,
        canvas: rgb(0x0021_222c),
        panel: background,
        hairline: rgb(0x0034_3746),
        card: background,
        card_hover_shadow: card_hover_shadow(true),
        text: foreground,
        // The comment color is too dim for secondary text, so muted sits between.
        muted: mix(comment, foreground, 0.45),
        faint: comment,
        accent: rgb(0x00bd_93f9),
        on_accent: rgb(0x0021_222c),
        soft: rgb(0x0034_3746),
        hover: rgb(0x0042_4450),
        ai: rgb(BRAND_RED),
        ai_text: ai_text(true),
        ai_soft: ai_soft(background, true),
        code_card: rgb(0x0019_1a21),
        code_text: foreground,
        danger: rgb(0x00ff_79c6),
        // The spec's colors and their bright ANSI shades, leaving out red and orange.
        folders: inks(
            [
                0x00f1_fa8c,
                0x0050_fa7b,
                0x0069_ff94,
                0x008b_e9fd,
                0x00a4_ffff,
                0x0062_72a4,
                0x00bd_93f9,
                0x00d6_acff,
            ]
            .map(rgb),
            pine(rgb(0x0050_fa7b), rgb(0x008b_e9fd), true),
            rgb(0x00ff_79c6),
            cocoa(rgb(0x00f1_fa8c), true),
            slate(rgb(0x0062_72a4), true),
        ),
    }
}

fn alucard() -> Theme {
    let background = rgb(0x00ff_fbeb);
    let comment = rgb(0x006c_664b);
    Theme {
        id: "alucard",
        name: "Alucard",
        dark: false,
        canvas: rgb(0x00ec_e9df),
        panel: background,
        hairline: rgb(0x00de_dccf),
        card: background,
        card_hover_shadow: card_hover_shadow(false),
        text: rgb(0x001f_1f1f),
        muted: comment,
        // Alucard has no grey lighter than its comment color.
        faint: mix(comment, background, 0.4),
        accent: rgb(0x0064_4ac9),
        on_accent: background,
        soft: rgb(0x00ef_eddc),
        hover: rgb(0x00e2_deca),
        ai: rgb(BRAND_RED),
        ai_text: ai_text(false),
        ai_soft: ai_soft(background, false),
        code_card: rgb(0x0028_2a36),
        code_text: rgb(0x00f8_f8f2),
        danger: rgb(0x00a3_144d),
        folders: inks(
            [
                0x0084_6e15,
                0x009e_841a,
                0x0014_710a,
                0x0019_8d0c,
                0x0003_6a96,
                0x0004_7fb4,
                0x0064_4ac9,
                0x0078_62d0,
            ]
            .map(rgb),
            between(rgb(0x0019_8d0c), rgb(0x0003_6a96)),
            // Alucard's pink is nearly crimson, so Rose leans it toward the purple.
            between(rgb(0x0064_4ac9), rgb(0x00a3_144d)),
            cocoa(rgb(0x0084_6e15), false),
            slate(rgb(0x0003_6a96), false),
        ),
    }
}
