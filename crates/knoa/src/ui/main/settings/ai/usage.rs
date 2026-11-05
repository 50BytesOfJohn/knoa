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
        let note = form::note("No AI requests yet.", theme.muted);
        return vec![form::group("Usage", vec![note], theme)];
    }
    let recent_label = format!("Last {RECENT_DAYS} days");
    let mut rows = vec![
        totals_row(&recent_label, &sum(&report.recent), theme),
        totals_row("All time", &sum(&report.all_time), theme),
    ];
    if let Some(provider) = provider {
        rows.push(form::row(
            &format!("{}'s own records", provider.name()),
            Some("Every request, with the exact amount charged."),
            button("ai-activity", "Open", Tone::Soft, cx)
                .on_click(move |_, _, cx| cx.open_url(provider.activity_url())),
            theme,
        ));
    }
    let mut groups = vec![form::group("Usage", rows, theme)];
    if !report.recent.is_empty() {
        let models = report.recent.iter().map(|m| model_row(m, theme)).collect();
        let title = format!("By model, last {RECENT_DAYS} days");
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
    let mut parts = vec![count(totals.requests, "request", "requests")];
    if totals.unmetered < totals.requests {
        parts.push(format!(
            "{} tokens in, {} out",
            tokens(totals.input_tokens),
            tokens(totals.output_tokens)
        ));
    }
    if totals.unmetered > 0 {
        parts.push(format!("{} without token counts", totals.unmetered));
    }
    if totals.unpriced > 0 {
        parts.push(format!("{} without a price", totals.unpriced));
    }
    parts.join(" · ")
}

fn count(n: u64, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// Cents hide what a few bookmarks cost, so small amounts get more digits.
fn dollars(cost: f64) -> String {
    if cost > 0.0 && cost < 0.01 {
        format!("${cost:.4}")
    } else {
        format!("${cost:.2}")
    }
}

fn tokens(n: u64) -> String {
    match n {
        0..1_000 => n.to_string(),
        1_000..1_000_000 => format!("{:.1}K", n as f64 / 1e3),
        _ => format!("{:.1}M", n as f64 / 1e6),
    }
}
