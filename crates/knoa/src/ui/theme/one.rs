//! One Dark and One Light, from Atom's one-*-syntax and one-*-ui packages. Atom defines
//! the UI levels as lightness steps from the syntax background; the values here are
//! those steps worked out. Neither has a pink, so purple and crimson mark danger, and
//! folders fill in between the remaining hues.

use super::{BRAND_RED, Theme, ai_soft, ai_text, below, card_hover_shadow, hsl, rgb, spread};

pub(super) fn themes() -> Vec<Theme> {
    vec![light(), dark()]
}

fn dark() -> Theme {
    let panel = hsl(220., 13., 18.);
    let canvas = hsl(220., 13., 15.);
    Theme {
        id: "one-dark",
        name: "One Dark",
        dark: true,
        canvas,
        panel,
        hairline: hsl(220., 13., 10.8),
        card: panel,
        card_hover_shadow: card_hover_shadow(true),
        text: rgb(0x00ab_b2bf),
        muted: hsl(220., 9., 55.),
        faint: rgb(0x005c_6370),
        accent: rgb(0x0061_afef),
        on_accent: canvas,
        soft: hsl(220., 13., 21.),
        hover: hsl(220., 13., 24.),
        ai: rgb(BRAND_RED),
        ai_text: ai_text(true),
        ai_soft: ai_soft(panel, true),
        code_card: below(canvas),
        code_text: rgb(0x00ab_b2bf),
        danger: rgb(0x00c6_78dd),
        folders: spread(
            [
                0x00e5_c07b,
                0x0098_c379,
                0x0056_b6c2,
                0x0061_afef,
                0x00c6_78dd,
            ]
            .map(rgb),
        ),
    }
}

fn light() -> Theme {
    let panel = hsl(230., 1., 98.);
    Theme {
        id: "one-light",
        name: "One Light",
        dark: false,
        canvas: hsl(230., 1., 92.),
        panel,
        hairline: hsl(230., 1., 86.),
        card: gpui::white(),
        card_hover_shadow: card_hover_shadow(false),
        text: hsl(230., 8., 24.),
        muted: hsl(230., 6., 44.),
        faint: hsl(230., 4., 64.),
        accent: hsl(221., 87., 60.),
        on_accent: gpui::white(),
        soft: hsl(230., 1., 92.),
        hover: hsl(230., 1., 89.),
        ai: rgb(BRAND_RED),
        ai_text: ai_text(false),
        ai_soft: ai_soft(panel, false),
        code_card: hsl(220., 13., 18.),
        code_text: rgb(0x00ab_b2bf),
        danger: hsl(344., 84., 43.),
        folders: spread([
            hsl(119., 34., 47.),
            hsl(198., 99., 37.),
            hsl(221., 87., 60.),
            hsl(301., 63., 40.),
            hsl(344., 84., 43.),
        ]),
    }
}
