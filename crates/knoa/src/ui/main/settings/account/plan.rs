//! The Account page's plan, billing action, and usage summary.

use gpui::{AnyElement, App, IntoElement, StatefulInteractiveElement as _};
use jiff::Timestamp;
use knoa_core::protocol::{Account, BillingStatus, Organizer, Plan};

use super::form;
use crate::{
    format,
    i18n::t,
    services::cloud::CloudConfig,
    ui::{
        components::{self, Tone},
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
                t!("settings-account-start-trial")
            } else {
                t!("settings-account-upgrade")
            },
            config.pricing.clone(),
        ),
        Plan::Pro => (
            t!("settings-account-manage-subscription"),
            config.billing.clone(),
        ),
    };
    components::button(id, label, Tone::Soft, cx)
        .on_click(move |_, _, cx| cx.open_url(url.as_str()))
}

pub(super) fn usage_row(account: &Account, theme: &Theme) -> AnyElement {
    let (usage, limits) = (account.usage, account.limits);
    let (used, limit) = (usage.bookmarks, limits.bookmarks);
    let about = match account.organizer {
        Organizer::Device => t!("settings-account-usage-device", used = used, limit = limit),
        Organizer::Cloud => t!(
            "settings-account-usage-cloud",
            used = used,
            limit = limit,
            organized = (usage.organized_this_month),
            allowance = (limits.organized_per_month)
        ),
    };
    form::row(
        &t!("settings-account-usage"),
        Some(&about),
        gpui::Empty,
        theme,
    )
}

/// Sentences on the plan, when it renews or ends, and who organizes.
pub(super) fn description(account: &Account) -> String {
    let plan = match (
        account.plan,
        account.billing.as_ref().map(|billing| billing.status),
    ) {
        (Plan::Free, _) => t!("settings-account-plan-free"),
        (Plan::Pro, Some(BillingStatus::Trialing)) => t!("settings-account-plan-trial"),
        (Plan::Pro, _) => t!("settings-account-plan-pro"),
    };
    let mut sentences = vec![plan];
    if let Some(billing) = &account.billing
        && billing.status != BillingStatus::None
        && let Some(end) = billing
            .period_end
            .and_then(|end| Timestamp::from_second(end).ok())
    {
        let date = format::instant(end, format::DATE);
        sentences.push(if billing.cancel_at_period_end {
            t!("settings-account-plan-ends", date = date)
        } else if billing.status == BillingStatus::Trialing {
            t!("settings-account-plan-trial-ends", date = date)
        } else {
            t!("settings-account-plan-renews", date = date)
        });
    }
    sentences.push(match account.organizer {
        Organizer::Device => t!("settings-account-organized-on-device"),
        Organizer::Cloud => t!("settings-account-organized-in-cloud"),
    });
    sentences.join(" ")
}
