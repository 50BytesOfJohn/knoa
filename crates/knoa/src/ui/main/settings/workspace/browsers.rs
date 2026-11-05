//! The workspace page's browser sections: which profiles feed it and where its links open.

use gpui::{AnyElement, Context, Div, IntoElement as _, ParentElement as _, Styled as _, div};
use gpui_component::{
    radio::{Radio, RadioGroup},
    switch::Switch,
};
use knoa_browser::{Profile, ProfileId};

use super::{WorkspacePage, form};
use crate::ui::{components::blocked_browsers_notice, theme::Theme};

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
            rows.push(form::note(
                "No Chromium-based browser found. Knoa reads Helium, Chrome, Brave, Edge, Vivaldi and Chromium.",
                theme.muted,
            ));
        }
        if let Some(notice) = blocked_browsers_notice(&self.blocked, cx) {
            rows.push(div().p_2().child(notice).into_any_element());
        }
        rows.push(form::note(
            "With none on, links are only added by hand (⌘N). One browser can feed several workspaces.",
            theme.muted,
        ));
        form::group("Sync bookmarks from", rows, theme)
    }

    pub(super) fn opens_in_group(&self, theme: &Theme, cx: &mut Context<'_, WorkspacePage>) -> Div {
        let (choices, labels): (Vec<_>, Vec<_>) = self.open_choices().into_iter().unzip();
        // A saved profile that's gone from this Mac shows nothing picked; opening falls
        // back to the default browser.
        let selected = choices.iter().position(|c| *c == self.opens_in);
        let radios = labels
            .into_iter()
            .enumerate()
            .map(|(index, label)| Radio::new(("opens-in", index)).label(label));
        let group = RadioGroup::vertical("opens-in")
            .children(radios)
            .selected_index(selected)
            .on_change(cx.listener(move |this, index: &usize, _, cx| {
                if let Some(choice) = choices.get(*index) {
                    this.opens_in.clone_from(choice);
                    this.changed(cx);
                }
            }));
        let row = div().px_4().py_3().child(group).into_any_element();
        form::group("Open bookmarks in", vec![row], theme)
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
        let mut choices = vec![(None, "Default browser".to_owned())];
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
        [] => "Links added by hand only.".to_owned(),
        [one] => format!("Synced from {one}."),
        [rest @ .., last] => format!("Synced from {} and {last}.", rest.join(", ")),
    }
}
