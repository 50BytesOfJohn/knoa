//! The Account page: what Cloud mode is and signing in, or the signed-in account, its
//! plan, how sync is doing, and signing out.

use gpui::{
    AnyElement, Context, Div, Entity, IntoElement, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Window, div,
};
use jiff::{Timestamp, Zoned, tz::TimeZone};
use knoa_core::protocol::{Account, BillingStatus, Organizer, Plan};

use super::{form, updates::when};
use crate::{
    services::cloud::{Cloud, Phase, SyncStatus},
    ui::{
        components::{Tone, button, grouped},
        theme::Theme,
    },
};

const WHAT_IT_IS: &str = "Cloud mode syncs your library across your devices and with Knoa on the web. With Pro, Knoa also organizes your bookmarks in the cloud, with no key of your own.";
const PRIVACY: &str = "In Cloud mode your library is kept on Knoa's servers, which read your pages to organize them. It isn't end-to-end encrypted. Signed out, everything stays on this Mac.";

pub struct AccountPage {
    cloud: Entity<Cloud>,
}

impl AccountPage {
    pub fn new(cx: &mut Context<'_, AccountPage>) -> AccountPage {
        let cloud = Cloud::entity(cx);
        cx.observe(&cloud, |_, _, cx| cx.notify()).detach();
        AccountPage { cloud }
    }
}

impl Render for AccountPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let cloud = self.cloud.read(cx);
        let header = form::header("Account", "Sync your library with a Knoa account.", &theme);
        let mut page = div().flex().flex_col().gap_8().child(header);
        if cloud.config().is_none() {
            let mut rows = vec![form::note(
                "Cloud sync isn't available in this build.",
                theme.muted,
            )];
            if let Some(reason) = cloud.unavailable() {
                rows.push(form::problem_note(
                    format!("It couldn't start: {reason}"),
                    &theme,
                ));
            }
            return page.child(form::group("Cloud mode", rows, &theme));
        }
        page = match cloud.phase() {
            Phase::SignedOut | Phase::SigningIn(_) => page.child(self.signed_out(&theme, cx)),
            Phase::OtherAccount(_) => page.child(self.other_account(&theme, cx)),
            Phase::SignedIn(_) => page
                .child(self.account_group(&theme, cx))
                .child(self.sync_group(&theme, cx)),
        };
        page
    }
}

impl AccountPage {
    fn signed_out(&self, theme: &Theme, cx: &mut Context<'_, AccountPage>) -> Div {
        let cloud = self.cloud.read(cx);
        let waiting = matches!(cloud.phase(), Phase::SigningIn(_));
        let mut rows = vec![
            form::note(WHAT_IT_IS, theme.text),
            form::note(PRIVACY, theme.muted),
        ];
        if let Some(problem) = cloud.problem() {
            rows.push(form::problem_note(problem.to_owned(), theme));
        }
        rows.push(if waiting {
            form::row(
                "Finish signing in in your browser…",
                Some("Knoa opened the sign-in page in your browser."),
                button("cancel-sign-in", "Cancel", Tone::Soft, cx).on_click(
                    cx.listener(|this, _, _, cx| this.cloud.update(cx, Cloud::cancel_sign_in)),
                ),
                theme,
            )
        } else {
            form::row(
                "Knoa account",
                Some("Sign in or create an account in your browser."),
                button("sign-in", "Sign in", Tone::Primary, cx)
                    .on_click(cx.listener(|this, _, _, cx| this.cloud.update(cx, Cloud::sign_in))),
                theme,
            )
        });
        form::group("Cloud mode", rows, theme)
    }

    fn other_account(&self, theme: &Theme, cx: &mut Context<'_, AccountPage>) -> Div {
        let email = self.email(cx);
        let rows = vec![
            form::problem_note(
                "This library is synced with another Knoa account. Nothing syncs until you choose.",
                theme,
            ),
            form::row(
                &format!("Sync with {email} instead"),
                Some("The library is merged into this account. The other account keeps its copy."),
                button("replace-account", "Use this account", Tone::Primary, cx).on_click(
                    cx.listener(|this, _, _, cx| {
                        this.cloud.update(cx, Cloud::replace_other_account);
                    }),
                ),
                theme,
            ),
            self.sign_out_row(theme, cx),
        ];
        form::group("Account", rows, theme)
    }

    fn account_group(&self, theme: &Theme, cx: &mut Context<'_, AccountPage>) -> Div {
        let cloud = self.cloud.read(cx);
        let account = cloud.account().cloned();
        let config = cloud.config().cloned();
        let plan_row = match (&account, config) {
            (Some(account), Some(config)) => {
                let about = plan_description(account);
                let (label, url) = match account.plan {
                    Plan::Free => (
                        if account
                            .billing
                            .as_ref()
                            .is_some_and(|billing| billing.trial_available)
                        {
                            "Start free trial"
                        } else {
                            "Upgrade"
                        },
                        config.pricing,
                    ),
                    Plan::Pro => ("Manage subscription", config.billing),
                };
                form::row(
                    &self.email(cx),
                    Some(&about),
                    button("plan", label, Tone::Soft, cx)
                        .on_click(move |_, _, cx| cx.open_url(url.as_str())),
                    theme,
                )
            }
            _ => form::note(
                "Signed in. Asking the server about the account…",
                theme.muted,
            ),
        };
        let mut rows = vec![plan_row];
        rows.extend(account.as_ref().map(|account| usage_row(account, theme)));
        rows.push(self.sign_out_row(theme, cx));
        form::group("Account", rows, theme)
    }

    fn sync_group(&self, theme: &Theme, cx: &mut Context<'_, AccountPage>) -> Div {
        let cloud = self.cloud.read(cx);
        let status = match cloud.sync() {
            SyncStatus::Starting => "Connecting…".to_owned(),
            SyncStatus::UpToDate => "Up to date".to_owned(),
            SyncStatus::Syncing(changes) => format!("Syncing {changes} changes…"),
            SyncStatus::Offline => "Offline. Changes sync when Knoa can reach the server.".into(),
            SyncStatus::Error(err) => format!("Sync stopped: {err}"),
        };
        let last = match cloud.last_synced() {
            Some(at) => format!("Last synced {}", when(at, &Zoned::now())),
            None => "Not synced yet".to_owned(),
        };
        let mut rows = vec![form::row(&status, Some(&last), gpui::Empty, theme)];
        if let Some(notice) = cloud.notice() {
            rows.push(form::problem_note(notice.to_owned(), theme));
        }
        form::group("Sync", rows, theme)
    }

    fn sign_out_row(&self, theme: &Theme, cx: &mut Context<'_, AccountPage>) -> AnyElement {
        form::row(
            "Sign out",
            Some("Your library stays on this Mac. Sign in again to carry on syncing."),
            button("sign-out", "Sign out", Tone::Soft, cx)
                .on_click(cx.listener(|this, _, _, cx| this.cloud.update(cx, Cloud::sign_out))),
            theme,
        )
    }

    fn email(&self, cx: &Context<'_, AccountPage>) -> String {
        self.cloud
            .read(cx)
            .account()
            .and_then(|account| account.email.clone())
            .unwrap_or_else(|| "this account".to_owned())
    }
}

fn usage_row(account: &Account, theme: &Theme) -> AnyElement {
    let (usage, limits) = (account.usage, account.limits);
    let bookmarks = format!(
        "{} of {} bookmarks",
        grouped(usage.bookmarks),
        grouped(limits.bookmarks)
    );
    let about = match account.organizer {
        Organizer::Device => bookmarks,
        Organizer::Cloud => format!(
            "{bookmarks}, {} of {} organized this month",
            grouped(usage.organized_this_month),
            grouped(limits.organized_per_month)
        ),
    };
    form::row("Usage", Some(&about), gpui::Empty, theme)
}

fn plan_description(account: &Account) -> String {
    use std::fmt::Write as _;

    let mut about = match (
        account.plan,
        account.billing.as_ref().map(|billing| billing.status),
    ) {
        (Plan::Free, _) => "Free plan.".to_owned(),
        (Plan::Pro, Some(BillingStatus::Trialing)) => "Pro trial.".to_owned(),
        (Plan::Pro, _) => "Pro plan.".to_owned(),
    };
    if let Some(billing) = &account.billing
        && billing.status != BillingStatus::None
        && let Some(end) = billing
            .period_end
            .and_then(|end| Timestamp::from_second(end).ok())
    {
        let date = end.to_zoned(TimeZone::system()).strftime("%b %-d, %Y");
        let action = if billing.cancel_at_period_end {
            "Ends"
        } else if billing.status == BillingStatus::Trialing {
            "Trial ends"
        } else {
            "Renews"
        };
        let _ = write!(about, " {action} {date}.");
    }
    about.push_str(match account.organizer {
        Organizer::Device => " Organizing uses your own AI key, on this Mac.",
        Organizer::Cloud => " Knoa organizes your bookmarks in the cloud.",
    });
    about
}
