//! The workspace page's browsers section: which profiles feed it and where its links open.

use gpui::{AnyElement, Context, Div, IntoElement as _, ParentElement as _, Styled as _, div};
use gpui_component::switch::Switch;
use knoa_browser::{Profile, ProfileId};

use super::{WorkspacePage, form};
use crate::{
    i18n::t,
    ui::{choice::select, components::blocked_browsers_notice, theme::Theme},
};

impl WorkspacePage {
    pub(super) fn browsers_group(&self, theme: &Theme, cx: &mut Context<'_, WorkspacePage>) -> Div {
        let mut rows: Vec<AnyElement> = Vec::new();
        for (index, profile) in self.profiles.iter().enumerate() {
            let id = profile.id.to_string();
            rows.push(form::row(
                profile.id.browser.name(),
                Some(&profile.name),
                Switch::new(("workspace-profile", index))
                    .checked(self.browsers.contains(&id))
                    .on_click(cx.listener(move |this, _: &bool, _, cx| {
                        this.toggle_browser(id.clone(), cx);
                    })),
                theme,
            ));
        }
        if self.profiles.is_empty() && self.blocked.is_empty() {
            rows.push(form::note(t!("settings-workspace-no-browser"), theme.muted));
        }
        if let Some(notice) = blocked_browsers_notice(&self.blocked, cx) {
            rows.push(div().p_2().child(notice).into_any_element());
        }
        rows.push(form::note(
            t!("settings-workspace-browsers-note"),
            theme.muted,
        ));
        rows.push(self.opens_in_row(theme, cx));
        form::group(&t!("settings-workspace-browsers"), rows, theme)
    }

    fn opens_in_row(&self, theme: &Theme, cx: &mut Context<'_, WorkspacePage>) -> AnyElement {
        let choices = self.open_choices();
        // A saved profile that's gone from this Mac shows nothing picked; opening falls
        // back to the default browser.
        let options: Vec<(usize, String)> = choices
            .iter()
            .enumerate()
            .map(|(index, (_, label))| (index, label.clone()))
            .collect();
        let selected = choices.iter().position(|(c, _)| *c == self.opens_in);
        let page = cx.entity().downgrade();
        let picker = select(
            "opens-in",
            options,
            selected,
            false,
            theme,
            move |index, cx| {
                if let Some((choice, _)) = choices.get(index) {
                    let choice = choice.clone();
                    let _ = page.update(cx, |this, cx| {
                        this.opens_in = choice;
                        this.changed(cx);
                    });
                }
            },
        );
        form::row(
            &t!("settings-workspace-open-in"),
            Some(&t!("settings-workspace-open-in-about")),
            picker,
            theme,
        )
    }

    /// The default browser, every readable profile, and the blocked browsers' profiles this
    /// workspace already knows by id. Opening needs only the id, not read access, so a
    /// browser macOS blocks (Brave) can still be picked.
    fn open_choices(&self) -> Vec<(Option<String>, String)> {
        let readable = self
            .profiles
            .iter()
            .map(|profile| (profile.id.clone(), profile_label(profile)));
        let known_blocked = self
            .browsers
            .iter()
            .chain(&self.opens_in)
            .filter_map(|id| ProfileId::parse(id))
            .filter(|id| self.blocked.contains(&id.browser))
            .map(|id| {
                let label = format!("{} · {}", id.browser.name(), id.dir);
                (id, label)
            });
        let mut choices = vec![(None, t!("settings-workspace-default-browser"))];
        for (id, label) in readable.chain(known_blocked) {
            let id = Some(id.to_string());
            if !choices.iter().any(|(c, _)| *c == id) {
                choices.push((id, label));
            }
        }
        choices
    }
}

/// "Helium · Work".
fn profile_label(profile: &Profile) -> String {
    format!("{} · {}", profile.id.browser.name(), profile.name)
}

/// "Synced from Helium and Chrome", or that links are only added by hand.
pub(super) fn synced_from(browsers: &[String]) -> String {
    let mut names: Vec<&str> = browsers
        .iter()
        .filter_map(|id| ProfileId::parse(id))
        .map(|id| id.browser.name())
        .collect();
    names.dedup();
    match names.as_slice() {
        [] => t!("settings-workspace-by-hand"),
        [one] => t!("settings-workspace-synced-from-one", browser = (*one)),
        [rest @ .., last] => t!(
            "settings-workspace-synced-from-many",
            browsers = (rest.join(", ")),
            last = (*last)
        ),
    }
}
