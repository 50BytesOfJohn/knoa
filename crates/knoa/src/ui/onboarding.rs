//! First-run setup: pick a browser, optionally connect AI, name the library. Every step
//! can be skipped, and one click skips the whole thing with defaults.

mod steps;

use gpui::{
    Animation, AnimationExt as _, AppContext as _, Context, Entity, FocusHandle, FontWeight,
    InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Task, TaskExt as _, Window, div, ease_out_quint,
    prelude::FluentBuilder as _, px,
};
use gpui_component::input::{InputState, TextareaState};
use knoa_ai::ProviderKind;
use knoa_browser::{Browser, Profile};
use knoa_core::Ink;

use crate::{
    services::ai,
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
    blocked: Vec<Browser>,
    chosen: Vec<String>,
    key: Entity<InputState>,
    key_check: KeyCheck,
    name: Entity<InputState>,
    color: Ink,
    instructions: Entity<TextareaState>,
    /// Holds focus so Tab can reach the Hello step's sign-in link.
    focus: FocusHandle,
    sign_in: FocusHandle,
    /// Setup was skipped to sign in, so the main screen opens on the Account page.
    wants_account: bool,
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
        let workspace = library.read(cx).workspace().cloned();
        let chosen = match &workspace {
            Some(w) if !w.browsers.is_empty() => w.browsers.clone(),
            _ => default_profiles(&profiles),
        };
        let key = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Paste your key: sk-or-v1-…")
                .masked(true)
        });
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("Personal"));
        if let Some(workspace) = &workspace {
            name.update(cx, |input, cx| {
                input.set_value(workspace.name.clone(), window, cx);
            });
        }
        let instructions = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("Optional. Write it like you'd tell a friend.")
        });
        if let Some(workspace) = &workspace {
            instructions.update(cx, |input, cx| {
                input.set_value(workspace.instructions.clone(), window, cx);
            });
        }
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        Onboarding {
            library,
            step: Step::Hello,
            profiles,
            blocked: knoa_browser::blocked_browsers(),
            chosen,
            key,
            key_check: KeyCheck::Idle,
            name,
            color: workspace.map_or(Ink::default(), |w| w.color),
            instructions,
            focus,
            sign_in: cx.focus_handle().tab_stop(true),
            wants_account: false,
        }
    }

    /// Whether setup was skipped to sign in to an existing account.
    pub fn wants_account(&self) -> bool {
        self.wants_account
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

    /// Saves everything chosen so far into the current workspace, which starts syncing.
    /// Shown on the last step, which stays up until the user opens the library.
    fn apply(&mut self, cx: &mut Context<'_, Onboarding>) {
        let name = self.name.read(cx).value().trim().to_owned();
        let instructions = self.instructions.read(cx).value().trim().to_owned();
        let (browsers, color) = (self.chosen.clone(), self.color);
        self.library.update(cx, |library, cx| {
            let id = library.workspace_id();
            library.change(cx, move |store| {
                let Some(mut workspace) = store.workspace(id)? else {
                    return Ok(());
                };
                workspace.name = if name.is_empty() {
                    "Personal".into()
                } else {
                    name
                };
                workspace.color = color;
                workspace.instructions = instructions;
                workspace.browsers = browsers;
                store.update_workspace(&workspace)
            });
        });
        if matches!(self.key_check, KeyCheck::Passed) {
            let key = self.key.read(cx).value().trim().to_owned();
            ai::enable(ProviderKind::OpenRouter, &key, cx).detach_and_log_err(cx);
        }
    }

    /// One click to the library with defaults: the detected browser, no AI.
    fn skip_all(&mut self, cx: &mut Context<'_, Onboarding>) {
        self.key_check = KeyCheck::Idle;
        self.apply(cx);
        self.finish(cx);
    }

    /// Skips setup as "Use the defaults" does, then leaves for the Account page.
    fn sign_in(&mut self, cx: &mut Context<'_, Onboarding>) {
        self.wants_account = true;
        self.skip_all(cx);
    }

    /// Tab moves between the Hello step's tab stops. Other steps leave Tab to their
    /// inputs.
    fn tab(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<'_, Self>) {
        if self.step != Step::Hello || event.keystroke.key != "tab" {
            return;
        }
        if event.keystroke.modifiers.shift {
            window.focus_prev(cx);
        } else {
            window.focus_next(cx);
        }
        cx.stop_propagation();
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
        .find(|(p, _)| p.id.browser == Browser::Helium)
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
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::tab))
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
