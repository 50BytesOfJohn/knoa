//! The AI step: an OpenRouter key and checking that it works.

use gpui::{
    Context, Div, InteractiveElement as _, ParentElement as _, StatefulInteractiveElement as _,
    Styled as _, div, px,
};
use gpui_component::input::Input;
use knoa_ai::ProviderKind;

use super::{lead, title};
use crate::ui::{
    components::{Tone, button, problem},
    mascot::{Mood, mascot},
    onboarding::{KeyCheck, Onboarding, Step},
    theme::Theme,
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
            KeyCheck::Checking(_) => Some(note().child("Knocking on OpenRouter's door…")),
            KeyCheck::Failed(message) => Some(problem(message.clone(), theme)),
            KeyCheck::Passed => Some(
                note()
                    .child(mascot(
                        "key-works",
                        px(28.),
                        Mood::Happy,
                        theme.ai,
                        theme.panel,
                    ))
                    .child("That key works."),
            ),
        }
    }

    pub(super) fn ai(&mut self, cx: &mut Context<'_, Onboarding>) -> Div {
        let theme = Theme::get(cx).clone();
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(title("Want me to do the tidying?"))
            .child(lead(
                "With an OpenRouter key I'll read each new page and give it a kind, a summary, tags and a folder. \
                 It's your key and your account: about 17 cents per thousand bookmarks.",
                &theme,
            ))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().flex_1().child(Input::new(&self.key)))
                    .child(button("check-key", "Check key", Tone::Primary, cx).on_click(cx.listener(|this, _, window, cx| this.check_key(window, cx)))),
            )
            .children(self.key_message(&theme))
            .child(
                div()
                    .flex()
                    .gap_3()
                    .text_sm()
                    .child(link("get-key", "Get a key ↗", &theme).on_click(|_, _, cx| cx.open_url(ProviderKind::OpenRouter.keys_url())))
                    .child(div().text_color(theme.faint).child("·"))
                    .child(div().text_color(theme.muted).child("Without one, I still fetch images and descriptions.")),
            )
            .child(
                div()
                    .mt_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(button("back", "Back", Tone::Ghost, cx).on_click(cx.listener(|this, _, window, cx| this.go(this.step.back(), window, cx))))
                    .child(div().flex_1())
                    .child(button("no-ai", "Skip, no AI", Tone::Ghost, cx).on_click(cx.listener(|this, _, window, cx| {
                        this.key_check = KeyCheck::Idle;
                        this.go(Step::Library, window, cx);
                    }))),
            )
    }
}

fn link(id: &'static str, text: &'static str, theme: &Theme) -> gpui::Stateful<Div> {
    div()
        .id(id)
        .text_color(theme.accent)
        .cursor_pointer()
        .hover(gpui::Styled::underline)
        .child(text)
}
