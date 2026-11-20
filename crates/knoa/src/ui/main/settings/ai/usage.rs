//! The AI page's usage groups: totals for the recent window and all time, then the recent
//! window per model.

use gpui::{
    AnyElement, App, Div, FontWeight, ParentElement as _, StatefulInteractiveElement as _,
    Styled as _, div,
};
use knoa_ai::ProviderKind;
use knoa_core::{ModelUsage, UsageTotals};

use super::super::form;
use crate::{
    format,
    i18n::t,
    services::usage::{RECENT_DAYS, UsageReport},
    ui::{
        components::{Tone, button},
        theme::Theme,
    },
};

/// `provider` is the one connected, for a link to its own records.
pub fn groups(
    report: &UsageReport,
    provider: Option<ProviderKind>,
    theme: &Theme,
    cx: &mut App,
) -> Vec<Div> {
    if report.is_empty() {
        let note = form::note(t!("settings-ai-usage-none"), theme.muted);
        return vec![form::group(&t!("settings-ai-usage"), vec![note], theme)];
    }
    let recent_label = t!("settings-ai-usage-recent", days = RECENT_DAYS);
    let mut rows = vec![
        totals_row(&recent_label, &sum(&report.recent), theme),
        totals_row(
            &t!("settings-ai-usage-all-time"),
            &sum(&report.all_time),
            theme,
        ),
    ];
    if let Some(provider) = provider {
        rows.push(form::row(
            &t!("settings-ai-usage-records", provider = (provider.name())),
            Some(&t!("settings-ai-usage-records-about")),
            button("ai-activity", t!("settings-ai-usage-open"), Tone::Soft, cx)
                .on_click(move |_, _, cx| cx.open_url(provider.activity_url())),
            theme,
        ));
    }
    let mut groups = vec![form::group(&t!("settings-ai-usage"), rows, theme)];
    if !report.recent.is_empty() {
        let models = report.recent.iter().map(|m| model_row(m, theme)).collect();
        let title = t!("settings-ai-usage-by-model", days = RECENT_DAYS);
        groups.push(form::group(&title, models, theme));
    }
    groups
}

fn sum(models: &[ModelUsage]) -> UsageTotals {
    let mut totals = UsageTotals::default();
    for model in models {
        totals.add(&model.totals);
    }
    totals
}

fn totals_row(label: &str, totals: &UsageTotals, theme: &Theme) -> AnyElement {
    form::row(label, Some(&details(totals)), amount(totals, theme), theme)
}

fn model_row(model: &ModelUsage, theme: &Theme) -> AnyElement {
    let totals = &model.totals;
    form::row(
        &model.model,
        Some(&details(totals)),
        amount(totals, theme),
        theme,
    )
}

fn amount(totals: &UsageTotals, theme: &Theme) -> Div {
    // A total missing some prices is a lower bound; "+" says so without a footnote.
    let more = if totals.unpriced > 0 { "+" } else { "" };
    div()
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.text)
        .child(format!("{}{more}", dollars(totals.cost)))
}

/// Requests and tokens, and what the provider left unreported.
fn details(totals: &UsageTotals) -> String {
    let mut parts = vec![t!("settings-ai-usage-requests", count = (totals.requests))];
    if totals.unmetered < totals.requests {
        parts.push(t!(
            "settings-ai-usage-tokens",
            input = (format::compact(totals.input_tokens)),
            output = (format::compact(totals.output_tokens))
        ));
    }
    if totals.unmetered > 0 {
        parts.push(t!(
            "settings-ai-usage-unmetered",
            count = (totals.unmetered)
        ));
    }
    if totals.unpriced > 0 {
        parts.push(t!("settings-ai-usage-unpriced", count = (totals.unpriced)));
    }
    parts.join(" · ")
}

/// Cents hide what a few bookmarks cost, so small amounts get more digits.
fn dollars(cost: f64) -> String {
    let fraction_digits = if cost > 0.0 && cost < 0.01 { 4 } else { 2 };
    format::usd(cost, fraction_digits)
}
