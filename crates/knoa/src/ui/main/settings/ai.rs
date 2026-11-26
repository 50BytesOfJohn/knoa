//! The AI page: connecting OpenRouter or using Pro, the models in use, and what AI has
//! cost. Organizing existing bookmarks is on the Organize page.

mod key;
mod pro;
mod usage;

use gpui::{
    App, AppContext as _, Context, Div, Entity, IntoElement, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Task, Window, div, px,
};
use gpui_component::input::InputState;

use super::{account::plan, form};
use crate::{
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
use key::KeyCheck;

pub struct AiPage {
    library: Entity<Library>,
    key: Entity<InputState>,
    /// Where the last key check got to.
    key_check: Option<KeyCheck>,
    checking: Option<Task<()>>,
    /// `None` until the first load, so the page doesn't flash "no requests".
    usage: Option<UsageReport>,
    loading_usage: Option<Task<()>>,
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
        let mut page = AiPage {
            library,
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
            AiStatus::Ready | AiStatus::NeedsAttention(_) => {
                page.child(self.provider_group(&status, &theme, cx))
            }
            AiStatus::Cloud => page.child(form::group(
                &t!("settings-ai-provider"),
                vec![form::note(cloud_note(cx), theme.muted)],
                &theme,
            )),
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
}

/// Where the cloud's organizing comes from, and what the key on this Mac is still for.
fn cloud_note(cx: &App) -> String {
    let cloud = Cloud::entity(cx);
    match cloud.read(cx).account().and_then(plan::free_allowance) {
        Some(allowance) => t!("settings-ai-cloud-note-free", allowance = (allowance)),
        None => t!("settings-ai-cloud-note"),
    }
}
