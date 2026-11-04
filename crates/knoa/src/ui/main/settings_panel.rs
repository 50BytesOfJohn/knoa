//! Settings: appearance and AI. Browsers and instructions belong to each workspace.

use gpui::{
    AppContext as _, Context, Entity, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _, Task,
    TaskExt as _, WeakEntity, Window, div, px,
};
use gpui_component::input::{Input, InputState};
use knoa_ai::ProviderKind;

use super::MainView;
use crate::{
    services::{
        ai,
        organizer::{AiStatus, Organizer},
    },
    state::{library::Library, settings::SettingsFile},
    ui::{
        components::{Tone, button},
        theme::{SERIF, Theme},
    },
};

/// About $0.17 per 1,000 bookmarks with the default models (enrichment research).
const DOLLARS_PER_BOOKMARK: f64 = 0.000_17;

pub struct SettingsPanel {
    library: Entity<Library>,
    main: WeakEntity<MainView>,
    key: Entity<InputState>,
    key_message: Option<(String, bool)>,
    checking: Option<Task<()>>,
}

impl SettingsPanel {
    pub fn new(
        library: Entity<Library>,
        main: WeakEntity<MainView>,
        window: &mut Window,
        cx: &mut Context<'_, SettingsPanel>,
    ) -> SettingsPanel {
        let key = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("sk-or-…")
                .masked(true)
        });
        cx.observe(&Organizer::entity(cx), |_, _, cx| cx.notify())
            .detach();
        SettingsPanel {
            library,
            main,
            key,
            key_message: None,
            checking: None,
        }
    }

    fn connect(&mut self, cx: &mut Context<'_, SettingsPanel>) {
        let key = self.key.read(cx).value().trim().to_owned();
        if key.is_empty() {
            return;
        }
        self.key_message = Some(("Checking the key…".into(), false));
        let check = ai::check_key(ProviderKind::OpenRouter, key.clone(), cx);
        self.checking = Some(cx.spawn(async move |this, cx| {
            let result = check.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(()) => {
                        ai::enable(ProviderKind::OpenRouter, &key, cx).detach_and_log_err(cx);
                        this.key_message = Some((
                            "Connected. New bookmarks get organized from now on.".into(),
                            false,
                        ));
                    }
                    Err(err) => {
                        this.key_message = Some((format!("That didn't work: {err}."), true));
                    }
                }
                cx.notify();
            });
        }));
        cx.notify();
    }

    fn organize_existing(&mut self, cx: &mut Context<'_, SettingsPanel>) {
        self.library.update(cx, |library, cx| {
            let workspace = library.workspace_id();
            library.change(cx, move |store| {
                store.queue_not_organized(workspace).map(drop)
            });
        });
        Organizer::wake(cx);
    }

    fn close(&self, window: &mut Window, cx: &mut Context<'_, SettingsPanel>) {
        let _ = self
            .main
            .update(cx, |main, cx| main.close_overlay(window, cx));
    }
}

impl Render for SettingsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        div()
            .id("settings-panel")
            .w(px(440.))
            .max_h(px(640.))
            .overflow_y_scroll()
            .p_5()
            .flex()
            .flex_col()
            .gap_5()
            .child(
                div()
                    .font_family(SERIF)
                    .text_xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Settings"),
            )
            .child(self.appearance(cx))
            .child(self.ai_section(cx))
            .child(self.workspace_section(cx))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        button("rerun-setup", "Run setup again", Tone::Ghost, cx).on_click(
                            cx.listener(|this, _, window, cx| {
                                SettingsFile::update(cx, |s| s.onboarded = false);
                                this.close(window, cx);
                                cx.refresh_windows();
                            }),
                        ),
                    )
                    .child(div().flex_1())
                    .child(
                        button("close-settings", "Done", Tone::Primary, cx)
                            .on_click(cx.listener(|this, _, window, cx| this.close(window, cx))),
                    ),
            )
    }
}

impl SettingsPanel {
    fn appearance(&self, cx: &mut Context<'_, SettingsPanel>) -> impl IntoElement {
        let current = SettingsFile::get(cx).theme.clone();
        let theme = Theme::get(cx).clone();
        let mut row = div().flex().gap_2();
        for option in Theme::all() {
            let id = option.id;
            let selected = current == id;
            row = row.child(
                div()
                    .id(SharedString::from(format!("theme-{id}")))
                    .flex_1()
                    .p_2()
                    .flex()
                    .flex_col()
                    .gap_1p5()
                    .rounded(px(12.))
                    .border_1()
                    .border_color(if selected {
                        theme.accent
                    } else {
                        theme.hairline
                    })
                    .cursor_pointer()
                    .child(
                        div()
                            .h(px(36.))
                            .rounded(px(8.))
                            .bg(option.canvas)
                            .p_1()
                            .flex()
                            .gap_1()
                            .child(div().w(px(14.)).rounded(px(4.)).bg(option.panel))
                            .child(div().flex_1().rounded(px(4.)).bg(option.card))
                            .child(div().w(px(6.)).rounded(px(3.)).bg(option.accent)),
                    )
                    .child(div().text_xs().text_center().child(option.name))
                    .on_click(cx.listener(move |_, _, _, cx| {
                        SettingsFile::update(cx, |s| s.theme = id.into());
                        Theme::apply(id, cx);
                    })),
            );
        }
        section("Appearance", &theme).child(row)
    }

    fn ai_section(&self, cx: &mut Context<'_, SettingsPanel>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let status = Organizer::entity(cx).read(cx).status().clone();
        let not_organized = self.library.read(cx).counts().not_organized;
        let mut body = div().flex().flex_col().gap_3();
        match &status {
            AiStatus::Off => {
                body = body
                    .child(div().text_sm().text_color(theme.muted).child(
                        "Knoa works without AI: pages are still fetched for images and descriptions. Add an OpenRouter key to sort, summarize and tag bookmarks.",
                    ))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(div().flex_1().child(Input::new(&self.key)))
                            .child(button("connect-ai", "Connect", Tone::Primary, cx).on_click(cx.listener(|this, _, _, cx| this.connect(cx)))),
                    )
                    .child(
                        button("get-key", "Get a key at openrouter.ai ↗", Tone::Ghost, cx)
                            .on_click(|_, _, cx| cx.open_url(ProviderKind::OpenRouter.keys_url())),
                    );
            }
            AiStatus::Ready | AiStatus::NeedsAttention(_) => {
                let models = SettingsFile::get(cx)
                    .ai
                    .as_ref()
                    .map(|c| c.chat_models.join(", "))
                    .unwrap_or_default();
                body = body.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .child(div().text_sm().child("✦ Connected to OpenRouter"))
                                .child(div().text_xs().text_color(theme.muted).child(models)),
                        )
                        .child(
                            button("disconnect-ai", "Disconnect", Tone::Ghost, cx).on_click(
                                cx.listener(|_, _, _, cx| {
                                    ai::disable(cx);
                                    cx.notify();
                                }),
                            ),
                        ),
                );
                if let AiStatus::NeedsAttention(problem) = &status {
                    body = body.child(div().text_sm().text_color(theme.danger).child(format!(
                        "OpenRouter says: {problem}. Check your key or credits."
                    )));
                }
                if not_organized > 0 {
                    let cost = not_organized as f64 * DOLLARS_PER_BOOKMARK;
                    body = body.child(
                        div()
                            .p_3()
                            .flex()
                            .items_center()
                            .gap_3()
                            .rounded(px(12.))
                            .bg(theme.ai_soft)
                            .child(div().flex_1().text_sm().child(format!("{not_organized} bookmarks in this workspace aren't organized yet. About ${cost:.2} to do them all.")))
                            .child(button("organize-all", "Organize", Tone::Primary, cx).on_click(cx.listener(|this, _, _, cx| this.organize_existing(cx)))),
                    );
                }
            }
        }
        if let Some((message, is_error)) = &self.key_message {
            body = body.child(
                div()
                    .text_sm()
                    .text_color(if *is_error { theme.danger } else { theme.muted })
                    .child(message.clone()),
            );
        }
        section("AI", &theme).child(body)
    }

    fn workspace_section(&self, cx: &mut Context<'_, SettingsPanel>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let workspace = self.library.read(cx).workspace().cloned();
        let name = workspace
            .as_ref()
            .map(|w| w.name.clone())
            .unwrap_or_default();
        let id = workspace.map(|w| w.id);
        section("Workspace", &theme).child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(div().flex_1().text_sm().text_color(theme.muted).child(format!(
                    "Browsers, organizing instructions and what to keep are set per workspace, here for {name}."
                )))
                .child(button("edit-workspace", "Edit workspace", Tone::Soft, cx).on_click(
                    cx.listener(move |this, _, window, cx| {
                        let _ = this
                            .main
                            .update(cx, |main, cx| main.edit_workspace(id, window, cx));
                    }),
                )),
        )
    }
}

fn section(title: &str, theme: &Theme) -> gpui::Div {
    div().flex().flex_col().gap_2().child(
        div()
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme.faint)
            .child(title.to_owned()),
    )
}
