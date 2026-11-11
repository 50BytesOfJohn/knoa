//! Everforest, dark and light at medium contrast, from sainnhe/everforest. Its red is
//! the brand's, so danger takes the pinkish purple, and folders have four hues, the
//! midpoints between them and the purple as Rose.

use gpui::Hsla;

use super::{BRAND_RED, Theme, ai_text, card_hover_shadow, cocoa, inks, pine, rgb, slate, spread};

pub(super) fn themes() -> Vec<Theme> {
    vec![light(), dark()]
}

fn dark() -> Theme {
    Theme {
        id: "everforest-dark",
        name: "Everforest Dark",
        dark: true,
        canvas: rgb(0x0023_2a2e),
        panel: rgb(0x002d_353b),
        hairline: rgb(0x003d_484d),
        card: rgb(0x0034_3f44),
        card_hover_shadow: card_hover_shadow(true),
        text: rgb(0x00d3_c6aa),
        muted: rgb(0x009d_a9a0),
        faint: rgb(0x007a_8478),
        accent: rgb(0x00a7_c080),
        on_accent: rgb(0x0023_2a2e),
        soft: rgb(0x0034_3f44),
        hover: rgb(0x003d_484d),
        ai: rgb(BRAND_RED),
        ai_text: ai_text(true),
        ai_soft: rgb(0x0051_4045),
        // The hard-contrast background, a step below this one's canvas.
        code_card: rgb(0x001e_2326),
        code_text: rgb(0x00d3_c6aa),
        danger: rgb(0x00d6_99b6),
        folders: folders(
            [
                0x00db_bc7f,
                0x00a7_c080,
                0x0083_c092,
                0x007f_bbb3,
                0x00d6_99b6,
            ],
            true,
        ),
    }
}

fn light() -> Theme {
    Theme {
        id: "everforest-light",
        name: "Everforest Light",
        dark: false,
        canvas: rgb(0x00ef_ebd4),
        panel: rgb(0x00fd_f6e3),
        hairline: rgb(0x00e0_dcc7),
        card: rgb(0x00fd_f6e3),
        card_hover_shadow: card_hover_shadow(false),
        text: rgb(0x005c_6a72),
        muted: rgb(0x0082_9181),
        faint: rgb(0x00a6_b0a0),
        // Aqua, not green: white text is easier to read on it.
        accent: rgb(0x0035_a77c),
        on_accent: rgb(0x00fd_f6e3),
        soft: rgb(0x00f4_f0d9),
        hover: rgb(0x00ef_ebd4),
        ai: rgb(BRAND_RED),
        ai_text: ai_text(false),
        ai_soft: rgb(0x00fd_e3da),
        code_card: rgb(0x002d_353b),
        code_text: rgb(0x00d3_c6aa),
        danger: rgb(0x00df_69ba),
        folders: folders(
            [
                0x00df_a000,
                0x008d_a101,
                0x0035_a77c,
                0x003a_94c5,
                0x00df_69ba,
            ],
            false,
        ),
    }
}

/// Yellow, green, aqua, blue and purple.
fn folders(hues: [u32; 5], dark: bool) -> [Hsla; 12] {
    let hues = hues.map(rgb);
    let [yellow, green, aqua, blue, purple] = hues;
    inks(
        spread(hues),
        pine(green, aqua, dark),
        purple,
        cocoa(yellow, dark),
        slate(blue, dark),
    )
}
