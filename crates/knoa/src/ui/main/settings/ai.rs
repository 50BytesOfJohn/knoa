//! The AI page: connecting OpenRouter, the models in use, organizing the bookmarks that
//! came in before AI was on, and what AI has cost.

mod pro;
mod usage;

use gpui::{
    AnyElement, AppContext as _, Context, Div, Entity, IntoElement, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Task, TaskExt as _, Window, div, px,
};
use gpui_component::input::{Input, InputState};
use knoa_ai::ProviderKind;

use super::{
    form,
    reorganize::{Reorganize, Scope},
};
use crate::{
    format,
    i18n::t,
    services::{
        ai,
        cloud::Cloud,
        organizer::{AiStatus, Organizer},
        usage::{self as usage_service, UsageReport},
    },
    state::{
        library::{Library, LibraryChanged},
        settings::SettingsFile,
    },
    ui::{
        components::{Tone, button},
        mascot::{Mood, mascot},
        theme::Theme,
    },
};

/// About $0.17 per 1,000 bookmarks with the default models (enrichment research).
const DOLLARS_PER_BOOKMARK: f64 = 0.000_17;

pub struct AiPage {
    library: Entity<Library>,
    reorganize: Entity<Reorganize>,
    key: Entity<InputState>,
    /// Where the last key check got to.
    key_check: Option<KeyCheck>,
    checking: Option<Task<()>>,
    /// `None` until the first load, so the page doesn't flash "no requests".
    usage: Option<UsageReport>,
    loading_usage: Option<Task<()>>,
}

enum KeyCheck {
    Checking,
    Connected,
    Failed(String),
}

impl AiPage {
    pub fn new(
        library: Entity<Library>,
        window: &mut Window,
        cx: &mut Context<'_, AiPage>,
    ) -> AiPage {
        let key = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("sk-or-…")
                .masked(true)
        });
        cx.observe(&Organizer::entity(cx), |_, _, cx| cx.notify())
            .detach();
        cx.observe(&library, |_, _, cx| cx.notify()).detach();
        cx.observe(&Cloud::entity(cx), |_, _, cx| cx.notify())
            .detach();
        // The library refreshes after every organized bookmark, which is when usage grows.
        cx.subscribe(&library, |this, _, _: &LibraryChanged, cx| {
            this.load_usage(cx);
        })
        .detach();
        let reorganize = cx.new(|_| Reorganize::new(Scope::AllWorkspaces, library.clone()));
        let mut page = AiPage {
            library,
            reorganize,
            key,
            key_check: None,
            checking: None,
            usage: None,
            loading_usage: None,
        };
        page.load_usage(cx);
        page
    }

    fn load_usage(&mut self, cx: &mut Context<'_, AiPage>) {
        let load = usage_service::load(self.library.read(cx).database(), cx);
        self.loading_usage = Some(cx.spawn(async move |this, cx| {
            let report = load.await;
            let _ = this.update(cx, |this, cx| match report {
                Ok(report) => {
                    this.usage = Some(report);
                    cx.notify();
                }
                Err(err) => tracing::error!("can't load AI usage: {err}"),
            });
        }));
    }

    fn connect(&mut self, cx: &mut Context<'_, AiPage>) {
        let key = self.key.read(cx).value().trim().to_owned();
        if key.is_empty() {
            return;
        }
        self.key_check = Some(KeyCheck::Checking);
        let check = ai::check_key(ProviderKind::OpenRouter, key.clone(), cx);
        self.checking = Some(cx.spawn(async move |this, cx| {
            let result = check.await;
            let _ = this.update(cx, |this, cx| {
                this.key_check = Some(match result {
                    Ok(()) => {
                        ai::enable(ProviderKind::OpenRouter, &key, cx).detach_and_log_err(cx);
                        KeyCheck::Connected
                    }
                    Err(err) => KeyCheck::Failed(err.to_string()),
                });
                cx.notify();
            });
        }));
        cx.notify();
    }

    fn organize_existing(&mut self, cx: &mut Context<'_, AiPage>) {
        self.library.update(cx, |library, cx| {
            let workspace = library.workspace_id();
            library.change(cx, move |store| {
                store.queue_not_organized(workspace).map(drop)
            });
        });
        Organizer::wake(cx);
    }
}

impl Render for AiPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let status = Organizer::entity(cx).read(cx).status().clone();
        let organizing = self.library.read(cx).counts().organizing;
        let header = div()
            .flex()
            .items_center()
            .gap_6()
            .child(div().flex_1().child(form::header(
                t!("settings-ai-title"),
                &t!("settings-ai-about"),
                &theme,
            )))
            .child(mascot(
                "ai-page-mascot",
                px(48.),
                Mood::of(&status, organizing),
                theme.ai,
                theme.panel,
            ));
        let mut page = div().flex().flex_col().gap_8().child(header);
        if status != AiStatus::Cloud {
            page = page.children(pro::card(&theme, cx));
        }
        page = match &status {
            AiStatus::Off => page.child(self.connect_group(&theme, cx)),
            AiStatus::Ready | AiStatus::NeedsAttention(_) => page
                .child(self.provider_group(&status, &theme, cx))
                .child(self.backlog_group(&theme, cx)),
            AiStatus::Cloud => page
                .child(form::group(
                    &t!("settings-ai-provider"),
                    vec![form::note(t!("settings-ai-cloud-note"), theme.muted)],
                    &theme,
                ))
                .child(self.backlog_group(&theme, cx)),
        };
        // Usage from before a disconnect still counts, so it stays visible while AI is off.
        let connected = SettingsFile::get(cx).ai.as_ref().map(|c| c.kind);
        if let Some(report) = &self.usage
            && (connected.is_some() || !report.is_empty())
        {
            page = page.children(usage::groups(report, connected, &theme, cx));
        }
        page
    }
}

impl AiPage {
    fn connect_group(&self, theme: &Theme, cx: &mut Context<'_, AiPage>) -> Div {
        let field = div()
            .flex()
            .gap_2()
            .child(div().flex_1().child(Input::new(&self.key)))
            .child(
                button("connect-ai", t!("settings-ai-connect"), Tone::Primary, cx)
                    .on_click(cx.listener(|this, _, _, cx| this.connect(cx))),
            );
        let mut rows = vec![form::stacked(
            &t!("settings-ai-key"),
            Some(&t!("settings-ai-key-about")),
            field,
            theme,
        )];
        rows.extend(self.message_row(theme));
        rows.push(form::row(
            &t!("settings-ai-no-key"),
            Some(&t!("settings-ai-no-key-about")),
            button("get-key", t!("settings-ai-get-key"), Tone::Soft, cx)
                .on_click(|_, _, cx| cx.open_url(ProviderKind::OpenRouter.keys_url())),
            theme,
        ));
        form::group(&t!("settings-ai-provider"), rows, theme)
    }

    fn provider_group(
        &self,
        status: &AiStatus,
        theme: &Theme,
        cx: &mut Context<'_, AiPage>,
    ) -> Div {
        let models = SettingsFile::get(cx)
            .ai
            .as_ref()
            .map(|c| c.chat_models.join(", "))
            .unwrap_or_default();
        let mut rows = vec![form::row(
            "OpenRouter",
            Some(&t!("settings-ai-connected")),
            button(
                "disconnect-ai",
                t!("settings-ai-disconnect"),
                Tone::Soft,
                cx,
            )
            .on_click(cx.listener(|_, _, _, cx| {
                ai::disable(cx);
                cx.notify();
            })),
            theme,
        )];
        if let AiStatus::NeedsAttention(problem) = status {
            rows.push(form::problem_note(
                t!("settings-ai-openrouter-says", problem = problem),
                theme,
            ));
        }
        rows.extend(self.message_row(theme));
        rows.push(form::row(
            &t!("settings-ai-models"),
            Some(&models),
            div(),
            theme,
        ));
        form::group(&t!("settings-ai-provider"), rows, theme)
    }

    fn backlog_group(&self, theme: &Theme, cx: &mut Context<'_, AiPage>) -> Div {
        let library = self.library.read(cx);
        let not_organized = library.counts().not_organized;
        let name = library
            .workspace()
            .map(|w| w.name.clone())
            .unwrap_or_default();
        let row = if not_organized == 0 {
            form::note(
                t!("settings-ai-all-organized", workspace = name),
                theme.muted,
            )
        } else {
            let cost = not_organized as f64 * DOLLARS_PER_BOOKMARK;
            let about = if *Organizer::entity(cx).read(cx).status() == AiStatus::Cloud {
                t!("settings-ai-included-in-pro")
            } else {
                t!("settings-ai-cost", cost = (format::usd(cost, 2)))
            };
            form::row(
                &t!(
                    "settings-ai-not-organized",
                    count = (not_organized),
                    workspace = name
                ),
                Some(&about),
                button(
                    "organize-all",
                    t!("settings-ai-organize"),
                    Tone::Primary,
                    cx,
                )
                .on_click(cx.listener(|this, _, _, cx| this.organize_existing(cx))),
                theme,
            )
        };
        let mut rows = vec![row];
        if library.workspaces().len() > 1 {
            rows.push(self.reorganize.clone().into_any_element());
        }
        form::group(&t!("settings-ai-existing"), rows, theme)
    }

    fn message_row(&self, theme: &Theme) -> Option<AnyElement> {
        Some(match self.key_check.as_ref()? {
            KeyCheck::Checking => form::note(t!("settings-ai-checking"), theme.muted),
            KeyCheck::Connected => form::note(t!("settings-ai-key-connected"), theme.muted),
            KeyCheck::Failed(err) => {
                form::problem_note(t!("settings-ai-key-failed", error = err), theme)
            }
        })
    }
}
