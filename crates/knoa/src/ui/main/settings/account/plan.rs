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

pub(in crate::ui::main::settings) fn button(
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

/// How many bookmarks a free account gets organized in the cloud each month. `None` for
/// Pro, and from servers that predate the free allowance.
pub(in crate::ui::main::settings) fn free_allowance(account: &Account) -> Option<u64> {
    let allowance = account.limits.organized_per_month;
    (account.plan == Plan::Free && allowance > 0).then_some(allowance)
}

/// What the cloud's organizing costs the account, for where the UI offers it.
pub(in crate::ui::main::settings) fn cloud_included(account: Option<&Account>) -> String {
    match account.and_then(free_allowance) {
        Some(allowance) => t!("settings-ai-included-in-free", allowance = (allowance)),
        None => t!("settings-ai-included-in-pro"),
    }
}

pub(super) fn usage_row(account: &Account, theme: &Theme) -> AnyElement {
    let (usage, limits) = (account.usage, account.limits);
    let (used, limit) = (usage.bookmarks, limits.bookmarks);
    // A free account that used its allowance is organized on the Mac again, but how much
    // of it went is still what it needs to know.
    let counts_organizing =
        account.organizer == Organizer::Cloud || free_allowance(account).is_some();
    let about = if counts_organizing {
        t!(
            "settings-account-usage-cloud",
            used = used,
            limit = limit,
            organized = (usage.organized_this_month),
            allowance = (limits.organized_per_month)
        )
    } else {
        t!("settings-account-usage-device", used = used, limit = limit)
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
    sentences.push(match (account.organizer, free_allowance(account)) {
        (Organizer::Device, _) => t!("settings-account-organized-on-device"),
        (Organizer::Cloud, Some(allowance)) => {
            t!("settings-account-organized-free", allowance = (allowance))
        }
        (Organizer::Cloud, None) => t!("settings-account-organized-in-cloud"),
    });
    sentences.join(" ")
}
