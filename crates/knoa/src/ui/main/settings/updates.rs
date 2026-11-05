//! The Updates group on the General page: the version, a manual check, and Sparkle's two
//! preferences.

use gpui::{AnyElement, App, Context, Div, StatefulInteractiveElement as _};
use gpui_component::{Disableable as _, switch::Switch};
use jiff::{Timestamp, Zoned, tz::TimeZone};

use super::{SettingsView, form};
use crate::{
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
            form::row(&format!("Knoa {VERSION}"), None, gpui::Empty, &theme),
            form::note(
                "This build doesn't update itself. Release builds from GitHub do.",
                theme.muted,
            ),
        ],
    };
    form::group("Updates", rows, &theme)
}

fn rows(updates: &Updates, theme: &Theme, cx: &App) -> Vec<AnyElement> {
    let checked = match updates.last_checked() {
        Some(at) => format!("Last checked {}", when(at, &Zoned::now())),
        None => "Not checked yet".to_owned(),
    };
    let checks = updates.checks_automatically();
    vec![
        form::row(
            &format!("Knoa {VERSION}"),
            Some(&checked),
            button("check-updates", "Check Now", Tone::Soft, cx).on_click(|_, _, cx| {
                if let Some(updates) = Updates::get(cx) {
                    updates.check_now();
                }
            }),
            theme,
        ),
        form::row(
            "Check for updates automatically",
            Some("Once a day, in the background."),
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
            "Install updates automatically",
            Some("Download new versions and install them when you quit Knoa."),
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

/// "today at 14:05", "yesterday at 09:30", or "on 3 Nov 2026".
fn when(at: Timestamp, now: &Zoned) -> String {
    let at = at.to_zoned(TimeZone::system());
    let days = (now.date() - at.date()).get_days();
    match days {
        0 => format!("today at {}", at.strftime("%H:%M")),
        1 => format!("yesterday at {}", at.strftime("%H:%M")),
        _ => format!("on {}", at.strftime("%-d %b %Y")),
    }
}
