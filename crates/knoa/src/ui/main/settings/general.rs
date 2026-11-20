//! The General page: the language, where the library lives, running setup again, updates.

use gpui::{
    AppContext as _, Context, Div, IntoElement as _, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div,
};

use super::{SettingsView, form, reorganize::Reorganize, updates};
use crate::{
    i18n::t,
    paths::AppPaths,
    services::cloud::Cloud,
    state::settings::SettingsFile,
    ui::{
        components::{Tone, button},
        language_picker::{Look, language_picker},
        theme::Theme,
    },
};

pub(super) fn page(view: &SettingsView, cx: &mut Context<'_, SettingsView>) -> Div {
    let theme = Theme::get(cx).clone();
    div()
        .flex()
        .flex_col()
        .gap_8()
        .child(form::header(
            t!("settings-general-title"),
            &t!("settings-general-about"),
            &theme,
        ))
        .child(language_group(view, &theme, cx))
        .child(library_group(&theme, cx))
        .child(setup_group(&theme, cx))
        .child(updates::group(cx))
}

/// The language, and once the user changes it here, what that leaves in the old one.
fn language_group(view: &SettingsView, theme: &Theme, cx: &mut Context<'_, SettingsView>) -> Div {
    let settings_view = cx.entity().downgrade();
    let picker = language_picker(
        "settings-language",
        SettingsFile::get(cx).language,
        Look::Field,
        theme,
        move |language, cx| {
            Cloud::choose_language(language, cx);
            let _ = settings_view.update(cx, SettingsView::show_language_note);
        },
    );
    let mut rows = vec![form::row(
        &t!("settings-general-language-label"),
        Some(&t!("settings-general-language-about")),
        picker,
        theme,
    )];
    if let Some(note) = &view.language_note {
        rows.push(note.clone().into_any_element());
    }
    form::group(&t!("settings-general-language"), rows, theme)
}

impl SettingsView {
    fn show_language_note(&mut self, cx: &mut Context<'_, SettingsView>) {
        if self.language_note.is_none() {
            let library = self.library.clone();
            self.language_note = Some(cx.new(|cx| Reorganize::after_language_change(library, cx)));
            cx.notify();
        }
    }
}

fn library_group(theme: &Theme, cx: &mut Context<'_, SettingsView>) -> Div {
    let paths = &cx.global::<AppPaths>().0;
    let (support, database) = (paths.support.clone(), paths.database());
    let location = form::row(
        &t!("settings-general-location"),
        Some(&home_relative(&support)),
        button(
            "show-library",
            t!("settings-general-show-in-finder"),
            Tone::Soft,
            cx,
        )
        .on_click(move |_, _, cx| cx.reveal_path(&database)),
        theme,
    );
    form::group(&t!("settings-general-library"), vec![location], theme)
}

fn setup_group(theme: &Theme, cx: &mut Context<'_, SettingsView>) -> Div {
    let setup = form::row(
        &t!("settings-general-run-setup-again"),
        Some(&t!("settings-general-run-setup-about")),
        button(
            "rerun-setup",
            t!("settings-general-run-setup"),
            Tone::Soft,
            cx,
        )
        .on_click(cx.listener(|this, _, window, cx| {
            SettingsFile::update(cx, |s| s.onboarded = false);
            this.close(window, cx);
            cx.refresh_windows();
        })),
        theme,
    );
    form::group(&t!("settings-general-setup"), vec![setup], theme)
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
