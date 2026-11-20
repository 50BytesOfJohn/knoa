//! The Updates group on the General page: the version, a manual check, and Sparkle's two
//! preferences.

use gpui::{AnyElement, App, Context, Div, StatefulInteractiveElement as _};
use gpui_component::{Disableable as _, switch::Switch};
use jiff::{Timestamp, Zoned, tz::TimeZone};

use super::{SettingsView, form};
use crate::{
    format,
    i18n::t,
    services::updates::Updates,
    ui::{
        components::{Tone, button},
        theme::Theme,
    },
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(super) fn group(cx: &mut Context<'_, SettingsView>) -> Div {
    let theme = Theme::get(cx).clone();
    let rows = match Updates::get(cx) {
        Some(updates) => rows(updates, &theme, cx),
        None => vec![
            form::row(
                &t!("settings-updates-version", version = VERSION),
                None,
                gpui::Empty,
                &theme,
            ),
            form::note(t!("settings-updates-self-built"), theme.muted),
        ],
    };
    form::group(&t!("settings-updates"), rows, &theme)
}

fn rows(updates: &Updates, theme: &Theme, cx: &App) -> Vec<AnyElement> {
    let checked = match updates.last_checked().map(|at| When::of(at, &Zoned::now())) {
        Some(When::Today(time)) => t!("settings-updates-checked-today", time = time),
        Some(When::Yesterday(time)) => t!("settings-updates-checked-yesterday", time = time),
        Some(When::On(date)) => t!("settings-updates-checked-on", date = date),
        None => t!("settings-updates-not-checked"),
    };
    let checks = updates.checks_automatically();
    vec![
        form::row(
            &t!("settings-updates-version", version = VERSION),
            Some(&checked),
            button(
                "check-updates",
                t!("settings-updates-check-now"),
                Tone::Soft,
                cx,
            )
            .on_click(|_, _, cx| {
                if let Some(updates) = Updates::get(cx) {
                    updates.check_now();
                }
            }),
            theme,
        ),
        form::row(
            &t!("settings-updates-check-automatically"),
            Some(&t!("settings-updates-check-automatically-about")),
            Switch::new("updates-check")
                .checked(checks)
                .on_click(|on: &bool, window, cx| {
                    if let Some(updates) = Updates::get(cx) {
                        updates.set_checks_automatically(*on);
                    }
                    window.refresh();
                }),
            theme,
        ),
        form::row(
            &t!("settings-updates-install-automatically"),
            Some(&t!("settings-updates-install-automatically-about")),
            Switch::new("updates-install")
                .checked(checks && updates.installs_automatically())
                .disabled(!checks)
                .on_click(|on: &bool, window, cx| {
                    if let Some(updates) = Updates::get(cx) {
                        updates.set_installs_automatically(*on);
                    }
                    window.refresh();
                }),
            theme,
        ),
    ]
}

/// When something last happened, for "Last checked …" and "Last synced …".
pub(super) enum When {
    /// At this time today.
    Today(String),
    Yesterday(String),
    /// On this date, before yesterday.
    On(String),
}

impl When {
    pub(super) fn of(at: Timestamp, now: &Zoned) -> When {
        let days = (now.date() - at.to_zoned(TimeZone::system()).date()).get_days();
        match days {
            0 => When::Today(format::instant(at, format::TIME)),
            1 => When::Yesterday(format::instant(at, format::TIME)),
            _ => When::On(format::instant(at, format::DATE)),
        }
    }
}
