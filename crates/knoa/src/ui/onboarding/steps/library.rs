//! The library step: the first workspace's name and color, and house rules for the AI.

use gpui::{
    Context, Div, FontWeight, InteractiveElement as _, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_component::input::{Input, Textarea};
use knoa_core::Ink;

use super::{nav, title};
use crate::ui::{
    components::workspace_mark,
    onboarding::{KeyCheck, Onboarding},
    theme::Theme,
};

/// Example instructions the user can add with a click.
const EXAMPLES: [&str; 4] = [
    "Keep work and personal things apart.",
    "Tag recipes by cuisine.",
    "Anything about Rust goes in a Rust folder.",
    "Short summaries, no fluff.",
];

impl Onboarding {
    pub(super) fn library_step(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Onboarding>,
    ) -> Div {
        let theme = Theme::get(cx).clone();
        let name = self.name.read(cx).value().to_string();
        let mut swatches = div().flex().gap_2();
        for ink in Ink::ALL {
            swatches = swatches.child(
                div()
                    .id(("color", ink.index()))
                    .size(px(22.))
                    .rounded(px(7.))
                    .bg(theme.ink(ink))
                    .cursor_pointer()
                    .when(ink == self.color, |s| s.border_2().border_color(theme.text))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.color = ink;
                        cx.notify();
                    })),
            );
        }
        let mut examples = div().flex().flex_wrap().gap_1p5();
        for (index, example) in EXAMPLES.iter().enumerate() {
            examples = examples.child(
                div()
                    .id(("example", index))
                    .px_2p5()
                    .h(px(26.))
                    .flex()
                    .items_center()
                    .rounded_full()
                    .bg(theme.soft)
                    .text_xs()
                    .text_color(theme.muted)
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.hover).text_color(theme.text))
                    .child(format!("+ {example}"))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        let current = this.instructions.read(cx).value().to_string();
                        let joined = if current.trim().is_empty() {
                            (*example).to_owned()
                        } else {
                            format!("{}\n{example}", current.trim_end())
                        };
                        this.instructions
                            .update(cx, |input, cx| input.set_value(joined, window, cx));
                    })),
            );
        }
        let ai_on = matches!(self.key_check, KeyCheck::Passed);
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(title("Last thing: what's this library called?"))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(workspace_mark(
                        if name.trim().is_empty() {
                            "Personal"
                        } else {
                            &name
                        },
                        self.color,
                        px(40.),
                        cx,
                    ))
                    .child(div().flex_1().child(Input::new(&self.name))),
            )
            .child(swatches)
            .when(ai_on, |s| {
                s.child(
                    div()
                        .mt_2()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .child("Any house rules for organizing?"),
                )
                .child(Textarea::new(&self.instructions).h(px(76.)))
                .child(examples)
            })
            .child(nav(self.step, "Finish", cx))
    }
}
