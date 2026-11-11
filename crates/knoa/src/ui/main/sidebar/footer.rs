//! The sidebar's footer: the wordmark with the mascot as its full stop, a word on what the
//! AI is doing, how cloud sync is doing when signed in, and the settings button.

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};

use super::{BUTTON_ICON, BUTTON_PADDING, CONTENT_INSET, TITLEBAR_BUTTON};
use crate::{
    services::{
        cloud::{Cloud, Phase, SyncStatus},
        organizer::AiStatus,
    },
    ui::{
        components::{icon, tip},
        main::{MainView, OpenSettings, settings::Page},
        mascot::{Mood, mascot},
        theme::{ROW_RADIUS, Theme},
        wordmark::wordmark_letters,
    },
};

impl MainView {
    pub(super) fn footer(
        &self,
        organizing: usize,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx);
        let status = self.organizer.read(cx).status().clone();
        let mood = Mood::of(&status, organizing);
        // Idle and ready needs no words: the mascot's face says it.
        let label = match &status {
            AiStatus::Off => Some(("AI off".to_owned(), theme.faint)),
            AiStatus::Ready | AiStatus::Cloud if organizing > 0 => {
                Some((organizing.to_string(), theme.ai_text))
            }
            AiStatus::Ready | AiStatus::Cloud => None,
            AiStatus::NeedsAttention(_) => Some(("Needs attention".to_owned(), theme.danger)),
        };
        // The mascot stands on the wordmark's baseline, where the logo's dot would be.
        let signature = div()
            .flex()
            .items_end()
            .gap(px(2.))
            .child(wordmark_letters(px(20.), theme.text))
            .child(
                div()
                    .mb(px(-2.))
                    // A new id per mood replays that mood's motion from its start.
                    .child(mascot(("footer-mascot", mood as usize), px(24.), mood, theme.ai, theme.panel)),
            );
        div()
            .flex_none()
            .h(px(56.))
            .pl(CONTENT_INSET)
            .pr(BUTTON_PADDING)
            .flex()
            .items_center()
            .gap_2()
            .border_t_1()
            .border_color(theme.hairline)
            .child(signature)
            .when_some(label, |s, (label, color)| {
                s.child(div().text_xs().text_color(color).child(label))
            })
            .child(div().flex_1())
            .children(self.sync_mark(cx))
            .child(
                div()
                    .id("settings")
                    .size(TITLEBAR_BUTTON)
                    .rounded(ROW_RADIUS)
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.hover))
                    .child(icon("settings", BUTTON_ICON, theme.muted))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.open_settings(&OpenSettings, window, cx);
                    })),
            )
    }
}

impl MainView {
    /// A quiet cloud while all is well, so a sync problem is noticed without a banner.
    fn sync_mark(&self, cx: &mut Context<'_, MainView>) -> Option<impl IntoElement> {
        let theme = Theme::get(cx);
        let cloud = Cloud::entity(cx);
        let cloud = cloud.read(cx);
        if !matches!(cloud.phase(), Phase::SignedIn(_) | Phase::OtherAccount(_)) {
            return None;
        }
        let (name, color, tooltip) = if matches!(cloud.phase(), Phase::OtherAccount(_)) {
            ("cloud-alert", theme.danger, "Sync is waiting for you")
        } else if cloud.notice().is_some() {
            ("cloud-alert", theme.danger, "Not everything could sync")
        } else {
            match cloud.sync() {
                SyncStatus::Starting | SyncStatus::Syncing(_) => {
                    ("refresh-cw", theme.faint, "Syncing")
                }
                SyncStatus::UpToDate => ("cloud-check", theme.faint, "Synced"),
                SyncStatus::Offline => ("cloud-off", theme.faint, "Offline"),
                SyncStatus::Error(_) => ("cloud-alert", theme.danger, "Sync stopped"),
            }
        };
        Some(
            div()
                .id("sync-status")
                .size(TITLEBAR_BUTTON)
                .rounded(ROW_RADIUS)
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(|s| s.bg(theme.hover))
                .child(icon(name, BUTTON_ICON, color))
                .tooltip(tip(tooltip, None, theme))
                .on_click(cx.listener(|this, _, window, cx| {
                    this.show_settings(Page::Account, window, cx);
                })),
        )
    }
}
