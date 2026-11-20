//! Who the sidebar shows in its footer and account menu: the signed-in account, or this
//! Mac when signed out, with the avatar that stands for it.

use gpui::{App, Div, FontWeight, ParentElement as _, Pixels, SharedString, Styled as _, div, px};
use knoa_core::{
    Ink,
    protocol::{Account, Plan},
};

use crate::{
    assets::line_icon,
    i18n::t,
    services::cloud::{Cloud, Phase},
    state::settings::SettingsFile,
    ui::{components::svg_icon, theme::Theme},
};

/// The account as shown, remade only when the account changes.
#[derive(Default)]
pub(super) struct Identity {
    made_for: Option<Signed<Account>>,
    who: Who,
}

/// Signed out, or signed in to the account the server named, once it has.
#[derive(Clone, Copy, PartialEq)]
enum Signed<A> {
    Out,
    In(Option<A>),
}

impl Signed<&Account> {
    fn matches(self, shown: &Signed<Account>) -> bool {
        match (self, shown) {
            (Signed::Out, Signed::Out) => true,
            (Signed::In(account), Signed::In(shown)) => account == shown.as_ref(),
            _ => false,
        }
    }
}

#[derive(Clone, Default)]
pub(super) enum Who {
    /// Signed out: the library lives on this Mac only.
    #[default]
    ThisMac,
    Account {
        /// The only name the server gives, empty when it gave none.
        email: SharedString,
        initials: SharedString,
        plan: Option<Plan>,
    },
}

impl Identity {
    pub fn who(&mut self, cx: &App) -> Who {
        let cloud = Cloud::entity(cx);
        let cloud = cloud.read(cx);
        let signed = match cloud.phase() {
            // The server answers a moment after sign-in; until then, the account last seen.
            Phase::SignedIn(_) | Phase::OtherAccount(_) => {
                Signed::In(cloud.account().or(SettingsFile::get(cx).account.as_ref()))
            }
            Phase::SignedOut | Phase::SigningIn(_) => Signed::Out,
        };
        if !self
            .made_for
            .as_ref()
            .is_some_and(|shown| signed.matches(shown))
        {
            let (who, shown) = match signed {
                Signed::Out => (Who::ThisMac, Signed::Out),
                Signed::In(account) => (Who::of(account), Signed::In(account.cloned())),
            };
            self.who = who;
            self.made_for = Some(shown);
        }
        self.who.clone()
    }
}

impl Who {
    fn of(account: Option<&Account>) -> Who {
        let email = account.and_then(|a| a.email.clone()).unwrap_or_default();
        Who::Account {
            initials: initials(&email).into(),
            email: email.into(),
            plan: account.map(|a| a.plan),
        }
    }

    pub fn name(&self) -> SharedString {
        match self {
            Who::ThisMac => t!("sidebar-this-mac").into(),
            Who::Account { email, .. } if email.is_empty() => t!("sidebar-knoa-account").into(),
            Who::Account { email, .. } => email.clone(),
        }
    }
}

/// Up to two letters from the email's name part: "tomasz.sz@…" gives "TS".
fn initials(email: &str) -> String {
    let local = email.split('@').next().unwrap_or_default();
    let mut words = local
        .split(['.', '_', '-', '+'])
        .filter_map(|word| word.chars().next());
    let initials: String = match (words.next(), words.next()) {
        (Some(first), Some(second)) => [first, second].into_iter().collect(),
        (Some(first), None) => first.to_string(),
        (None, _) => "K".to_owned(),
    };
    initials.to_uppercase()
}

/// The account's round avatar with its initials, or a rounded square with a laptop for
/// this Mac.
pub(super) fn avatar(who: &Who, size: Pixels, theme: &Theme) -> Div {
    let frame = div()
        .size(size)
        .flex_none()
        .flex()
        .items_center()
        .justify_center();
    match who {
        Who::ThisMac => frame
            .rounded(size * 0.3)
            .bg(theme.soft)
            .border_1()
            .border_color(theme.hairline)
            .child(svg_icon(line_icon!("laptop"), size * 0.58, theme.muted)),
        Who::Account { initials, .. } => frame
            .rounded_full()
            .bg(theme.soft.blend(theme.ink(Ink::Sky).opacity(0.45)))
            .text_color(theme.text)
            .text_size(size * 0.41)
            .font_weight(FontWeight::BOLD)
            .child(initials.clone()),
    }
}

/// The avatar's size in the footer.
pub(super) const FOOTER_AVATAR: Pixels = px(22.);
/// The avatar's size in the account menu's header.
pub(super) const MENU_AVATAR: Pixels = px(28.);
