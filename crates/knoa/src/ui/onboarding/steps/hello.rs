//! The first step: who Knoa is, and the way to skip setup.

use gpui::{
    Context, Div, InteractiveElement as _, KeyDownEvent, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};

use super::{lead, link, title};
use crate::services::cloud::Cloud;
use crate::ui::{
    components::{Tone, button},
    onboarding::{Onboarding, Step},
    theme::Theme,
    wordmark::written_wordmark,
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
            .child(title("Hi, I'm Knoa."))
            .child(lead(
                "You keep bookmarking things like a squirrel stocking up for winter. I find them again. \
                 I read your browser's bookmarks, work out what each one is, and keep the lot tidy.",
                &theme,
            ))
            .child(div().text_sm().text_color(theme.faint).child("Three quick questions. Or none, if you're in a hurry."))
            .child(
                div()
                    .mt_4()
                    .flex()
                    .gap_2()
                    .child(button("start", "Let's go", Tone::Primary, cx).px_6().on_click(cx.listener(|this, _, window, cx| this.go(Step::Browser, window, cx))))
                    .child(button("defaults", "Use the defaults", Tone::Ghost, cx).on_click(cx.listener(|this, _, _, cx| this.skip_all(cx)))),
            )
            .when(Cloud::entity(cx).read(cx).config().is_some(), |step| {
                step.child(self.sign_in_line(&theme, cx))
            })
    }

    /// For a new Mac and an existing account: skip setup and go straight to signing in.
    fn sign_in_line(&self, theme: &Theme, cx: &mut Context<'_, Onboarding>) -> Div {
        div()
            .flex()
            .gap_1()
            .text_sm()
            .text_color(theme.faint)
            .child("Already have a Knoa account?")
            .child(
                link("sign-in", "Sign in", theme)
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
