//! The sidebar's footer: the wordmark with the mascot as its full stop, a word on what the
//! AI is doing, and the settings button.

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};

use crate::{
    services::organizer::AiStatus,
    ui::{
        components::icon,
        main::{MainView, OpenSettings},
        mascot::{Mood, mascot},
        theme::Theme,
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
            AiStatus::Ready if organizing > 0 => Some((organizing.to_string(), theme.ai_text)),
            AiStatus::Ready => None,
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
            .h(px(52.))
            .px_4()
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
            .child(
                div()
                    .id("settings")
                    .size(px(28.))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.hover))
                    .child(icon("settings", px(16.), theme.muted))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.open_settings(&OpenSettings, window, cx);
                    })),
            )
    }
}
