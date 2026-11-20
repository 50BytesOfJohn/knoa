//! The first step: who Knoa is, and the way to skip setup.

use gpui::{
    Context, Div, InteractiveElement as _, KeyDownEvent, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};

use super::{lead, link, title};
use crate::{
    i18n::t,
    services::cloud::Cloud,
    state::settings::SettingsFile,
    ui::{
        components::{Tone, button},
        language_picker::{Look, language_picker},
        onboarding::{Onboarding, Step},
        theme::Theme,
        wordmark::written_wordmark,
    },
};

impl Onboarding {
    pub(super) fn hello(&mut self, cx: &mut Context<'_, Onboarding>) -> Div {
        let theme = Theme::get(cx).clone();
        div()
            .flex()
            .flex_col()
            .items_center()
            .text_center()
            .gap_4()
            .pt_6()
            .child(written_wordmark("hello-wordmark", px(84.), theme.text))
            .child(title(&t!("onboarding-hello-title")))
            .child(lead(&t!("onboarding-hello-lead"), &theme))
            .child(
                div()
                    .text_sm()
                    .text_color(theme.faint)
                    .child(t!("onboarding-hello-questions")),
            )
            .child(
                div()
                    .mt_4()
                    .flex()
                    .gap_2()
                    .child(
                        button("start", t!("onboarding-hello-start"), Tone::Primary, cx)
                            .px_6()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.go(Step::Browser, window, cx);
                            })),
                    )
                    .child(
                        button("defaults", t!("onboarding-hello-defaults"), Tone::Ghost, cx)
                            .on_click(cx.listener(|this, _, _, cx| this.skip_all(cx))),
                    ),
            )
            .when(Cloud::entity(cx).read(cx).config().is_some(), |step| {
                step.child(self.sign_in_line(&theme, cx))
            })
            .child(language_picker(
                "onboarding-language",
                SettingsFile::get(cx).language,
                Look::Compact,
                &theme,
                Cloud::choose_language,
            ))
    }

    /// For a new Mac and an existing account: skip setup and go straight to signing in.
    fn sign_in_line(&self, theme: &Theme, cx: &mut Context<'_, Onboarding>) -> Div {
        div()
            .flex()
            .gap_1()
            .text_sm()
            .text_color(theme.faint)
            .child(t!("onboarding-hello-have-account"))
            .child(
                link("sign-in", t!("onboarding-hello-sign-in"), theme)
                    .track_focus(&self.sign_in)
                    .focus_visible(gpui::Styled::underline)
                    .on_click(cx.listener(|this, _, _, cx| this.sign_in(cx)))
                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                        if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                            this.sign_in(cx);
                        }
                    })),
            )
    }
}
