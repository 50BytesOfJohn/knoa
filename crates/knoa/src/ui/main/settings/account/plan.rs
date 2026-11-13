//! The Account page's plan, billing action, and usage summary.

use gpui::{AnyElement, App, IntoElement, StatefulInteractiveElement as _};
use jiff::{Timestamp, tz::TimeZone};
use knoa_core::protocol::{Account, BillingStatus, Organizer, Plan};

use super::form;
use crate::{
    services::cloud::CloudConfig,
    ui::{
        components::{self, Tone, grouped},
        theme::Theme,
    },
};

pub(super) fn button(
    id: &'static str,
    account: &Account,
    config: &CloudConfig,
    cx: &App,
) -> impl IntoElement {
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
            config.pricing.clone(),
        ),
        Plan::Pro => ("Manage subscription", config.billing.clone()),
    };
    components::button(id, label, Tone::Soft, cx)
        .on_click(move |_, _, cx| cx.open_url(url.as_str()))
}

pub(super) fn usage_row(account: &Account, theme: &Theme) -> AnyElement {
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

pub(super) fn description(account: &Account) -> String {
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
