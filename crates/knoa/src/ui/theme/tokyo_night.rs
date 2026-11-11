//! Tokyo Night and Tokyo Night Storm, from folke/tokyonight.nvim. Its Day style is left
//! out: tokyonight computes it by inverting Night instead of publishing a palette.
//! Danger takes the hot pink, since both reds sit next to the brand red. Rose is the hot
//! pink softened toward magenta.

use super::{
    BRAND_RED, Theme, ai_soft, ai_text, between, card_hover_shadow, cocoa, inks, pine, rgb, slate,
};

pub(super) fn themes() -> Vec<Theme> {
    vec![
        theme(
            "tokyo-night",
            "Tokyo Night",
            0x001a_1b26,
            0x0016_161e,
            0x000c_0e14,
        ),
        theme(
            "tokyo-night-storm",
            "Tokyo Night Storm",
            0x0024_283b,
            0x001f_2335,
            0x001b_1e2d,
        ),
    ]
}

/// The styles share every color but the three backgrounds.
fn theme(id: &'static str, name: &'static str, bg: u32, bg_dark: u32, bg_dark1: u32) -> Theme {
    Theme {
        id,
        name,
        dark: true,
        canvas: rgb(bg_dark),
        panel: rgb(bg),
        hairline: rgb(0x003b_4261),
        card: rgb(bg),
        card_hover_shadow: card_hover_shadow(true),
        text: rgb(0x00c0_caf5),
        muted: rgb(0x00a9_b1d6),
        faint: rgb(0x0073_7aa2),
        accent: rgb(0x007a_a2f7),
        on_accent: rgb(bg_dark),
        soft: rgb(0x0029_2e42),
        hover: rgb(0x0029_2e42),
        ai: rgb(BRAND_RED),
        ai_text: ai_text(true),
        ai_soft: ai_soft(rgb(bg), true),
        code_card: rgb(bg_dark1),
        code_text: rgb(0x00c0_caf5),
        danger: rgb(0x00ff_007c),
        folders: inks(
            [
                0x00e0_af68,
                0x009e_ce6a,
                0x0073_daca,
                0x001a_bc9c,
                0x007d_cfff,
                0x007a_a2f7,
                0x009d_7cd8,
                0x00bb_9af7,
            ]
            .map(rgb),
            pine(rgb(0x009e_ce6a), rgb(0x0073_daca), true),
            between(rgb(0x00bb_9af7), rgb(0x00ff_007c)),
            cocoa(rgb(0x00e0_af68), true),
            slate(rgb(0x007a_a2f7), true),
        ),
    }
}
