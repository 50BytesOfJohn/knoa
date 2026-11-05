//! The General page: where the library lives, running setup again, updates.

use gpui::{Context, Div, ParentElement as _, StatefulInteractiveElement as _, Styled as _, div};

use super::{SettingsView, form, updates};
use crate::{
    paths::Paths,
    state::settings::SettingsFile,
    ui::{
        components::{Tone, button},
        theme::Theme,
    },
};

pub(super) fn page(cx: &mut Context<'_, SettingsView>) -> Div {
    let theme = Theme::get(cx).clone();
    let support = cx.global::<Paths>().support.clone();
    let database = cx.global::<Paths>().database();

    let location = form::row(
        "Library location",
        Some(&home_relative(&support)),
        button("show-library", "Show in Finder", Tone::Soft, cx)
            .on_click(move |_, _, cx| cx.reveal_path(&database)),
        &theme,
    );
    let setup = form::row(
        "Run setup again",
        Some("Pick browsers, AI and instructions from the start. Your bookmarks stay."),
        button("rerun-setup", "Run setup", Tone::Soft, cx).on_click(cx.listener(
            |this, _, window, cx| {
                SettingsFile::update(cx, |s| s.onboarded = false);
                this.close(window, cx);
                cx.refresh_windows();
            },
        )),
        &theme,
    );
    div()
        .flex()
        .flex_col()
        .gap_8()
        .child(form::header(
            "General",
            "Where Knoa keeps your library, how it started, and how it updates.",
            &theme,
        ))
        .child(form::group("Library", vec![location], &theme))
        .child(form::group("Setup", vec![setup], &theme))
        .child(updates::group(cx))
}

/// `~/Library/Application Support/Knoa` rather than the full path with the user's name.
fn home_relative(path: &std::path::Path) -> String {
    match dirs::home_dir()
        .and_then(|home| path.strip_prefix(home).ok().map(std::path::Path::to_owned))
    {
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
}
