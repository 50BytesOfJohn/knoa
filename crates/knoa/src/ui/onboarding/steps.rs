//! The onboarding steps' content.

mod ai;
mod browser;
mod done;
mod hello;
mod library;

use gpui::{
    AnyElement, Context, Div, FontWeight, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use super::{Onboarding, Step};
use crate::ui::{
    components::{Tone, button},
    theme::{SERIF, Theme},
};

impl Onboarding {
    pub(super) fn render_step(
        &mut self,
        window: &mut Window,
        cx: &mut Context<'_, Onboarding>,
    ) -> AnyElement {
        match self.step {
            Step::Hello => self.hello(cx),
            Step::Browser => self.browser(cx),
            Step::Ai => self.ai(cx),
            Step::Library => self.library_step(window, cx),
            Step::Done => self.done(cx),
        }
        .into_any_element()
    }
}

fn title(text: &str) -> Div {
    div()
        .font_family(SERIF)
        .text_size(px(30.))
        .line_height(px(36.))
        .font_weight(FontWeight::SEMIBOLD)
        .child(text.to_owned())
}

fn lead(text: &str, theme: &Theme) -> Div {
    div()
        .text_base()
        .line_height(px(23.))
        .text_color(theme.muted)
        .child(text.to_owned())
}

fn nav(step: Step, label: impl Into<SharedString>, cx: &mut Context<'_, Onboarding>) -> Div {
    div()
        .mt_2()
        .flex()
        .items_center()
        .gap_2()
        .child(
            button("back", "Back", Tone::Ghost, cx)
                .on_click(cx.listener(|this, _, window, cx| this.go(this.step.back(), window, cx))),
        )
        .child(div().flex_1())
        .child(
            button(("next", step as usize), label, Tone::Primary, cx)
                .px_5()
                .on_click(cx.listener(|this, _, window, cx| this.go(this.step.next(), window, cx))),
        )
}
