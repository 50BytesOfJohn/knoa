//! The Appearance page: the theme gallery and how the library shows cards.
//!
//! Themes are tiles that wrap, split into light and dark, so the gallery grows by rows
//! as themes are added and never needs a different control.

use gpui::{
    Context, Div, InteractiveElement as _, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};
use gpui_component::switch::Switch;

use super::{SettingsView, form};
use crate::{state::settings::SettingsFile, ui::theme::Theme};

const TILE_WIDTH: gpui::Pixels = px(132.);
const PREVIEW_HEIGHT: gpui::Pixels = px(84.);

pub(super) fn page(cx: &mut Context<'_, SettingsView>) -> Div {
    let theme = Theme::get(cx).clone();
    let current = SettingsFile::get(cx).theme.clone();
    let (dark, light): (Vec<Theme>, Vec<Theme>) = Theme::all().into_iter().partition(|t| t.dark);

    div()
        .flex()
        .flex_col()
        .gap_8()
        .child(form::header(
            "Appearance",
            "How Knoa looks. Changes show right away.",
            &theme,
        ))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_4()
                .child(form::group_title("Theme", &theme))
                .child(gallery("Light", &light, &current, &theme, cx))
                .child(gallery("Dark", &dark, &current, &theme, cx)),
        )
        .child(library_group(&theme, cx))
}

fn gallery(
    label: &str,
    themes: &[Theme],
    current: &str,
    theme: &Theme,
    cx: &mut Context<'_, SettingsView>,
) -> Div {
    let mut tiles = div().flex().flex_wrap().gap_x_4().gap_y_5();
    for option in themes {
        tiles = tiles.child(tile(option, option.id == current, theme, cx));
    }
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .px_1()
                .text_xs()
                .text_color(theme.muted)
                .child(label.to_owned()),
        )
        .child(tiles)
}

/// A small picture of the main screen in the theme's colours, with its name below.
fn tile(
    option: &Theme,
    selected: bool,
    theme: &Theme,
    cx: &mut Context<'_, SettingsView>,
) -> impl gpui::IntoElement {
    let id = option.id;
    // The ring sits in a fixed 2 px border, so selecting a tile never shifts the grid.
    let ring = if selected {
        theme.accent
    } else {
        gpui::transparent_black()
    };
    let hover_ring = if selected {
        theme.accent
    } else {
        theme.hairline
    };
    div()
        .id(SharedString::from(format!("theme-{id}")))
        .w(TILE_WIDTH)
        .flex()
        .flex_col()
        .gap_1p5()
        .cursor_pointer()
        .child(
            div()
                .p(px(2.))
                .rounded(px(12.))
                .border_2()
                .border_color(ring)
                .hover(move |s| s.border_color(hover_ring))
                .child(preview(option)),
        )
        .child(
            div()
                .px_1()
                .text_sm()
                .when(selected, |s| s.font_weight(gpui::FontWeight::MEDIUM))
                .when(!selected, |s| s.text_color(theme.muted))
                .child(option.name),
        )
        .on_click(cx.listener(move |_, _, _, cx| {
            SettingsFile::update(cx, |s| s.theme = id.into());
            Theme::apply(id, cx);
        }))
}

/// The sidebar with its selected place, and masonry cards of uneven heights.
fn preview(option: &Theme) -> Div {
    let bar = |width: f32, color| div().h(px(3.)).w(px(width)).rounded_full().bg(color);
    let card = |height: f32| div().h(px(height)).rounded(px(3.)).bg(option.card);
    let sidebar = div()
        .w(px(30.))
        .h_full()
        .p(px(5.))
        .flex()
        .flex_col()
        .gap(px(4.))
        .rounded(px(4.))
        .bg(option.panel)
        .child(bar(16., option.accent))
        .child(bar(12., option.faint))
        .child(bar(14., option.faint))
        .child(bar(10., option.faint));
    let column = |a: f32, b: f32| {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(card(a))
            .child(card(b))
    };
    div()
        .h(PREVIEW_HEIGHT)
        .p(px(6.))
        .flex()
        .gap(px(5.))
        .rounded(px(8.))
        .bg(option.canvas)
        .border_1()
        .border_color(option.hairline)
        .overflow_hidden()
        .child(sidebar)
        .child(column(34., 30.))
        .child(column(22., 42.))
        .child(column(40., 24.))
}

fn library_group(theme: &Theme, cx: &mut Context<'_, SettingsView>) -> Div {
    let settings = SettingsFile::get(cx).clone();
    let summaries = form::row(
        "Show summaries on cards",
        Some("Up to three lines under each title."),
        Switch::new("settings-show-summaries")
            .checked(settings.show_summaries)
            .on_click(cx.listener(|this, on: &bool, _, cx| {
                let on = *on;
                let _ = this
                    .main
                    .update(cx, |main, cx| main.set_show_summaries(on, cx));
            })),
        theme,
    );
    let up_next = form::row(
        "Show the Up next strip",
        Some("The row of things to read or watch, at the top of Everything."),
        Switch::new("settings-show-up-next")
            .checked(settings.show_up_next)
            .on_click(cx.listener(|_, on: &bool, _, cx| {
                let on = *on;
                SettingsFile::update(cx, |s| s.show_up_next = on);
                cx.refresh_windows();
            })),
        theme,
    );
    form::group("Library", vec![summaries, up_next], theme)
}
