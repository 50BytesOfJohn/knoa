//! First-run setup: pick a browser, optionally connect AI, name the library. Every step
//! can be skipped, and one click skips the whole thing with defaults.

mod steps;

use gpui::{
    Animation, AnimationExt as _, AppContext as _, Context, Entity, FontWeight,
    InteractiveElement as _, IntoElement, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Task, TaskExt as _, Window, div, ease_out_quint,
    prelude::FluentBuilder as _, px,
};
use gpui_component::input::{InputState, TextareaState};
use knoa_ai::ProviderKind;
use knoa_browser::Profile;

use crate::{
    services::{ai, sync},
    state::{library::Library, settings::SettingsFile},
    ui::theme::Theme,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Step {
    Hello,
    Browser,
    Ai,
    Library,
    Done,
}

impl Step {
    const ORDER: [Step; 5] = [
        Step::Hello,
        Step::Browser,
        Step::Ai,
        Step::Library,
        Step::Done,
    ];

    fn next(self) -> Step {
        let at = Step::ORDER.iter().position(|s| *s == self).unwrap_or(0);
        Step::ORDER.get(at + 1).copied().unwrap_or(Step::Done)
    }

    fn back(self) -> Step {
        let at = Step::ORDER.iter().position(|s| *s == self).unwrap_or(0);
        Step::ORDER[at.saturating_sub(1)]
    }
}

/// Where the key check stands.
enum KeyCheck {
    Idle,
    /// Holds the running check; dropping it cancels the check.
    Checking(#[allow(dead_code)] Task<()>),
    Failed(String),
    Passed,
}

pub struct Onboarding {
    library: Entity<Library>,
    step: Step,
    profiles: Vec<(Profile, usize)>,
    chosen: Vec<String>,
    key: Entity<InputState>,
    key_check: KeyCheck,
    name: Entity<InputState>,
    color: u8,
    instructions: Entity<TextareaState>,
}

impl Onboarding {
    pub fn new(
        library: Entity<Library>,
        window: &mut Window,
        cx: &mut Context<'_, Onboarding>,
    ) -> Onboarding {
        let profiles: Vec<(Profile, usize)> = knoa_browser::discover_profiles()
            .into_iter()
            .map(|p| {
                let count = knoa_browser::read_bookmarks(&p.bookmarks_path)
                    .map_or(0, |f| f.bookmarks.len());
                (p, count)
            })
            .collect();
        let chosen = default_profiles(&profiles);
        let settings = SettingsFile::get(cx).clone();
        let key = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Paste your key: sk-or-v1-…")
                .masked(true)
        });
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("Personal"));
        name.update(cx, |input, cx| {
            input.set_value(settings.workspace.name.clone(), window, cx);
        });
        let instructions = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("Optional. Write it like you'd tell a friend.")
        });
        instructions.update(cx, |input, cx| {
            input.set_value(settings.instructions.clone(), window, cx);
        });
        Onboarding {
            library,
            step: Step::Hello,
            profiles,
            chosen,
            key,
            key_check: KeyCheck::Idle,
            name,
            color: settings.workspace.color,
            instructions,
        }
    }

    fn go(&mut self, step: Step, window: &mut Window, cx: &mut Context<'_, Onboarding>) {
        self.step = step;
        match step {
            Step::Ai => self.key.update(cx, |input, cx| input.focus(window, cx)),
            Step::Library => self.name.update(cx, |input, cx| input.focus(window, cx)),
            Step::Done => self.apply(cx),
            _ => {}
        }
        cx.notify();
    }

    /// Checks the pasted key, then moves on.
    fn check_key(&mut self, window: &mut Window, cx: &mut Context<'_, Onboarding>) {
        let key = self.key.read(cx).value().trim().to_owned();
        if key.is_empty() {
            self.key_check = KeyCheck::Failed("Paste a key first, or skip this step.".into());
            cx.notify();
            return;
        }
        let check = ai::check_key(ProviderKind::OpenRouter, key, cx);
        let window_handle = window.window_handle();
        self.key_check = KeyCheck::Checking(cx.spawn(async move |this, cx| {
            let result = check.await;
            let _ = cx.update_window(window_handle, |_, window, cx| {
                let _ = this.update(cx, |this, cx| match result {
                    Ok(()) => {
                        this.key_check = KeyCheck::Passed;
                        this.go(Step::Library, window, cx);
                    }
                    Err(err) => {
                        this.key_check = KeyCheck::Failed(match err {
                            knoa_ai::Error::Unauthorized => {
                                "OpenRouter doesn't recognise that key. Double-check it?".into()
                            }
                            other => format!("Couldn't check the key: {other}"),
                        });
                        cx.notify();
                    }
                });
            });
        }));
        cx.notify();
    }

    /// Saves everything chosen so far and starts syncing. Shown on the last step, which
    /// stays up until the user opens the library.
    fn apply(&mut self, cx: &mut Context<'_, Onboarding>) {
        let name = self.name.read(cx).value().trim().to_owned();
        let instructions = self.instructions.read(cx).value().trim().to_owned();
        let (chosen, color) = (self.chosen.clone(), self.color);
        SettingsFile::update(cx, |s| {
            s.workspace.name = if name.is_empty() {
                "Personal".into()
            } else {
                name
            };
            s.workspace.color = color;
            s.instructions = instructions;
            s.browsers = chosen;
        });
        if matches!(self.key_check, KeyCheck::Passed) {
            let key = self.key.read(cx).value().trim().to_owned();
            ai::enable(ProviderKind::OpenRouter, &key, cx).detach_and_log_err(cx);
        }
        sync::restart(self.library.clone(), cx);
    }

    /// One click to the library with defaults: the detected browser, no AI.
    fn skip_all(&mut self, cx: &mut Context<'_, Onboarding>) {
        self.key_check = KeyCheck::Idle;
        self.apply(cx);
        self.finish(cx);
    }

    fn finish(&mut self, cx: &mut Context<'_, Onboarding>) {
        SettingsFile::update(cx, |s| s.onboarded = true);
        cx.refresh_windows();
    }
}

/// Helium first, as the browser Knoa is built around; otherwise the first profile found.
fn default_profiles(profiles: &[(Profile, usize)]) -> Vec<String> {
    let preferred = profiles
        .iter()
        .find(|(p, _)| p.id.browser == knoa_browser::Browser::Helium)
        .or_else(|| profiles.iter().max_by_key(|(_, count)| *count));
    preferred
        .map(|(p, _)| vec![p.id.to_string()])
        .unwrap_or_default()
}

impl Render for Onboarding {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let step = self.step;
        let content = self.render_step(window, cx);
        let dots = Step::ORDER.iter().fold(div().flex().gap_1p5(), |row, s| {
            row.child(
                div()
                    .h(px(6.))
                    .w(px(if *s == step { 22. } else { 6. }))
                    .rounded_full()
                    .bg(if *s <= step {
                        theme.accent
                    } else {
                        theme.hairline
                    }),
            )
        });
        div()
            .id("onboarding")
            .size_full()
            .relative()
            .bg(theme.canvas)
            .text_color(theme.text)
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(560.))
                    .min_h(px(460.))
                    .p_10()
                    .flex()
                    .flex_col()
                    .rounded(px(28.))
                    .bg(theme.panel)
                    .border_1()
                    .border_color(theme.hairline)
                    .shadow_lg()
                    .child(
                        div().flex_1().child(content).with_animation(
                            ("step", step as usize),
                            Animation::new(std::time::Duration::from_millis(380))
                                .with_easing(ease_out_quint()),
                            |el, t| el.opacity(t).mt(px(12. * (1. - t))),
                        ),
                    )
                    .child(div().mt_8().flex().justify_center().child(dots)),
            )
            .when(step != Step::Done, |s| {
                s.child(
                    div()
                        .id("skip-setup")
                        .absolute()
                        .top_4()
                        .right_5()
                        .px_3()
                        .h(px(30.))
                        .flex()
                        .items_center()
                        .rounded_full()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.muted)
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.soft).text_color(theme.text))
                        .child("Skip setup →")
                        .on_click(cx.listener(|this, _, _, cx| this.skip_all(cx))),
                )
            })
    }
}
