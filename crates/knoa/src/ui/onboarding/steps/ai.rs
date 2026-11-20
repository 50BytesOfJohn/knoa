//! The AI step: an OpenRouter key and checking that it works.

use gpui::{
    Context, Div, ParentElement as _, StatefulInteractiveElement as _, Styled as _, div, px,
};
use gpui_component::input::Input;
use knoa_ai::ProviderKind;

use super::{lead, link, title};
use crate::{
    i18n::t,
    ui::{
        components::{Tone, button, problem},
        mascot::{Mood, mascot},
        onboarding::{KeyCheck, Onboarding, Step},
        theme::Theme,
    },
};

impl Onboarding {
    /// How the key check went, if one ran.
    fn key_message(&self, theme: &Theme) -> Option<Div> {
        let note = || {
            div()
                .flex()
                .items_center()
                .gap_2()
                .text_sm()
                .text_color(theme.muted)
        };
        match &self.key_check {
            KeyCheck::Idle => None,
            KeyCheck::Checking(_) => Some(note().child(t!("onboarding-ai-checking"))),
            KeyCheck::Failed(why) => Some(problem(why.message(), theme)),
            KeyCheck::Passed => Some(
                note()
                    .child(mascot(
                        "key-works",
                        px(28.),
                        Mood::Happy,
                        theme.ai,
                        theme.panel,
                    ))
                    .child(t!("onboarding-ai-key-works")),
            ),
        }
    }

    pub(super) fn ai(&mut self, cx: &mut Context<'_, Onboarding>) -> Div {
        let theme = Theme::get(cx).clone();
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(title(&t!("onboarding-ai-title")))
            .child(lead(&t!("onboarding-ai-lead"), &theme))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().flex_1().child(Input::new(&self.key)))
                    .child(
                        button(
                            "check-key",
                            t!("onboarding-ai-check-key"),
                            Tone::Primary,
                            cx,
                        )
                        .on_click(cx.listener(|this, _, window, cx| this.check_key(window, cx))),
                    ),
            )
            .children(self.key_message(&theme))
            .child(
                div()
                    .flex()
                    .gap_3()
                    .text_sm()
                    .child(
                        link("get-key", t!("onboarding-ai-get-key"), &theme)
                            .on_click(|_, _, cx| cx.open_url(ProviderKind::OpenRouter.keys_url())),
                    )
                    .child(div().text_color(theme.faint).child("·"))
                    .child(
                        div()
                            .text_color(theme.muted)
                            .child(t!("onboarding-ai-without-key")),
                    ),
            )
            .child(
                div()
                    .mt_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        button("back", t!("onboarding-back"), Tone::Ghost, cx).on_click(
                            cx.listener(|this, _, window, cx| {
                                this.go(this.step.back(), window, cx);
                            }),
                        ),
                    )
                    .child(div().flex_1())
                    .child(
                        button("no-ai", t!("onboarding-ai-skip"), Tone::Ghost, cx).on_click(
                            cx.listener(|this, _, window, cx| {
                                this.key_check = KeyCheck::Idle;
                                this.go(Step::Library, window, cx);
                            }),
                        ),
                    ),
            )
    }
}
