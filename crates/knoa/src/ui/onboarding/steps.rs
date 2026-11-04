//! The onboarding steps' content.

use gpui::{
    Animation, AnimationExt as _, AnyElement, Context, Div, FontWeight, InteractiveElement as _,
    IntoElement, ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _,
    Window, div, prelude::FluentBuilder as _, pulsating_between, px,
};
use gpui_component::input::{Input, Textarea};
use knoa_ai::ProviderKind;

use super::{KeyCheck, Onboarding, Step};
use crate::{
    services::organizer::Organizer,
    ui::{
        components::{Tone, button, icon, workspace_mark},
        theme::{SERIF, Theme},
    },
};

/// Example instructions the user can add with a click.
const EXAMPLES: [&str; 4] = [
    "Keep work and personal things apart.",
    "Tag recipes by cuisine.",
    "Anything about Rust goes in a Rust folder.",
    "Short summaries, no fluff.",
];

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

    fn hello(&mut self, cx: &mut Context<'_, Onboarding>) -> Div {
        let theme = Theme::get(cx).clone();
        div()
            .flex()
            .flex_col()
            .items_center()
            .text_center()
            .gap_4()
            .pt_6()
            .child(
                div()
                    .text_size(px(44.))
                    .text_color(theme.ai)
                    .child("✦")
                    .with_animation("sparkle", Animation::new(std::time::Duration::from_millis(1800)).repeat().with_easing(pulsating_between(0.45, 1.)), gpui::Styled::opacity),
            )
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
    }

    fn browser(&mut self, cx: &mut Context<'_, Onboarding>) -> Div {
        let theme = Theme::get(cx).clone();
        let mut list = div().flex().flex_col().gap_2();
        for (index, (profile, count)) in self.profiles.iter().enumerate() {
            let id = profile.id.to_string();
            let on = self.chosen.contains(&id);
            let name = if profile.name == profile.id.dir {
                profile.id.browser.name().to_owned()
            } else {
                format!("{} · {}", profile.id.browser.name(), profile.name)
            };
            list = list.child(
                div()
                    .id(("profile", index))
                    .p_3()
                    .flex()
                    .items_center()
                    .gap_3()
                    .rounded(px(14.))
                    .border_1()
                    .border_color(if on { theme.accent } else { theme.hairline })
                    .bg(if on {
                        theme.accent.opacity(0.06)
                    } else {
                        theme.panel
                    })
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.soft))
                    .child(icon(
                        "globe",
                        px(20.),
                        if on { theme.accent } else { theme.muted },
                    ))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(div().text_sm().font_weight(FontWeight::MEDIUM).child(name))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted)
                                    .child(format!("{count} bookmarks")),
                            ),
                    )
                    .child(
                        div()
                            .size(px(20.))
                            .rounded_full()
                            .border_1()
                            .border_color(if on { theme.accent } else { theme.faint })
                            .bg(if on { theme.accent } else { theme.panel })
                            .flex()
                            .items_center()
                            .justify_center()
                            .when(on, |s| s.child(icon("check", px(13.), theme.on_accent))),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(at) = this.chosen.iter().position(|c| *c == id) {
                            this.chosen.remove(at);
                        } else {
                            this.chosen.push(id.clone());
                        }
                        cx.notify();
                    })),
            );
        }
        if self.profiles.is_empty() {
            list = list.child(
                div().p_4().rounded(px(14.)).bg(theme.soft).text_sm().text_color(theme.muted).child(
                    "I couldn't find a Chromium-based browser here (Helium, Chrome, Brave, Edge, Vivaldi). No worries: you can pick one later in Settings.",
                ),
            );
        }
        let total: usize = self
            .profiles
            .iter()
            .filter(|(p, _)| self.chosen.contains(&p.id.to_string()))
            .map(|(_, n)| n)
            .sum();
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(title("Where do your bookmarks live?"))
            .child(lead("Keep bookmarking the way you do. I'll notice new ones within a couple of seconds. I only read; I never touch your browser's bookmarks.", &theme))
            .child(list)
            .child(nav(self.step, if total > 0 { format!("Bring in {total}") } else { "Next".to_owned() }, cx))
    }

    fn ai(&mut self, cx: &mut Context<'_, Onboarding>) -> Div {
        let theme = Theme::get(cx).clone();
        let (message, error) = match &self.key_check {
            KeyCheck::Idle => (None, false),
            KeyCheck::Checking(_) => (Some("Knocking on OpenRouter's door…".to_owned()), false),
            KeyCheck::Failed(message) => (Some(message.clone()), true),
            KeyCheck::Passed => (Some("That key works. ✦".to_owned()), false),
        };
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
            .when_some(message, |s, message| s.child(div().text_sm().text_color(if error { theme.danger } else { theme.muted }).child(message)))
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

    fn library_step(&mut self, _window: &mut Window, cx: &mut Context<'_, Onboarding>) -> Div {
        let theme = Theme::get(cx).clone();
        let name = self.name.read(cx).value().to_string();
        let mut swatches = div().flex().gap_2();
        for index in 0..8u8 {
            swatches = swatches.child(
                div()
                    .id(("color", usize::from(index)))
                    .size(px(22.))
                    .rounded(px(7.))
                    .bg(theme.folder_color(index))
                    .cursor_pointer()
                    .when(index == self.color, |s| {
                        s.border_2().border_color(theme.text)
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.color = index;
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

    fn done(&mut self, cx: &mut Context<'_, Onboarding>) -> Div {
        let theme = Theme::get(cx).clone();
        let counts = self.library.read(cx).counts();
        let ai_on = matches!(self.key_check, KeyCheck::Passed);
        let line = match (counts.everything, self.chosen.is_empty()) {
            (_, true) => "No browser yet, so your library starts empty. Pick one in Settings whenever you like.".to_owned(),
            (0, false) => "Bringing your bookmarks in…".to_owned(),
            (n, false) => format!("{n} bookmarks are in, searchable right away."),
        };
        let mut body = div()
            .flex()
            .flex_col()
            .items_center()
            .text_center()
            .gap_4()
            .pt_6()
            .child(div().text_size(px(44.)).child("🎉"))
            .child(title("All set."))
            .child(lead(&line, &theme));
        if ai_on && counts.not_organized > 0 {
            let cost = counts.not_organized as f64 * 0.000_17;
            body = body.child(
                div()
                    .p_4()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .rounded(px(16.))
                    .bg(theme.ai_soft)
                    .child(div().text_sm().child(format!(
                        "New bookmarks get organized on their own. Want me to go through the {} you already have too? About ${cost:.2}.",
                        counts.not_organized
                    )))
                    .child(div().flex().justify_center().child(button("organize-existing", "Yes, organize them", Tone::Soft, cx).on_click(cx.listener(|this, _, _, cx| {
                        this.library.update(cx, |library, cx| {
                            let workspace = library.workspace_id();
                            library.change(cx, move |store| store.queue_not_organized(workspace).map(drop));
                        });
                        Organizer::wake(cx);
                        this.finish(cx);
                    })))),
            );
        }
        body.child(
            button("open-library", "Open my library", Tone::Primary, cx)
                .px_6()
                .on_click(cx.listener(|this, _, _, cx| this.finish(cx))),
        )
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

fn link(id: &'static str, text: &'static str, theme: &Theme) -> gpui::Stateful<Div> {
    div()
        .id(id)
        .text_color(theme.accent)
        .cursor_pointer()
        .hover(gpui::Styled::underline)
        .child(text)
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
