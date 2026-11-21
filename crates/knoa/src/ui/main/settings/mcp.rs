//! The MCP page: the connector URL AI apps use to reach the library, how to add it, and
//! how much each workspace lets them do. The settings live on the server, so they load
//! when the page shows.

use std::time::Duration;

use gpui::{
    AnyElement, ClipboardItem, Context, Div, Entity, EventEmitter, FontWeight, IntoElement,
    ParentElement as _, Render, StatefulInteractiveElement as _, Styled as _, Task, Window, div,
};
use knoa_core::{
    WorkspaceId,
    protocol::{McpAccess, Plan},
};

use super::{account::plan, form};
use crate::{
    i18n::t,
    services::cloud::{Cloud, McpState, Phase},
    state::library::Library,
    ui::{
        choice::{segmented, select},
        components::{Tone, button, icon},
        theme::{ROW_RADIUS, Theme},
    },
};

const ACCESS: [McpAccess; 3] = [McpAccess::Off, McpAccess::Read, McpAccess::Full];
/// How long Copy says Copied.
const COPIED_FOR: Duration = Duration::from_secs(2);

pub enum McpPageEvent {
    /// MCP goes through the account, which is set up on the Account page.
    OpenAccount,
}

impl EventEmitter<McpPageEvent> for McpPage {}

pub struct McpPage {
    cloud: Entity<Cloud>,
    library: Entity<Library>,
    /// Set while Copy says Copied; dropping it ends that early.
    copied: Option<Task<()>>,
}

impl McpPage {
    pub fn new(library: Entity<Library>, cx: &mut Context<'_, McpPage>) -> McpPage {
        let cloud = Cloud::entity(cx);
        cx.observe(&cloud, |_, _, cx| cx.notify()).detach();
        cx.observe(&library, |_, _, cx| cx.notify()).detach();
        McpPage {
            cloud,
            library,
            copied: None,
        }
    }

    pub fn load(&mut self, cx: &mut Context<'_, McpPage>) {
        self.cloud.update(cx, Cloud::load_mcp_settings);
    }

    fn copy(&mut self, url: String, cx: &mut Context<'_, McpPage>) {
        cx.write_to_clipboard(ClipboardItem::new_string(url));
        self.copied = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(COPIED_FOR).await;
            let _ = this.update(cx, |this, cx| {
                this.copied = None;
                cx.notify();
            });
        }));
        cx.notify();
    }
}

impl Render for McpPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        self.cloud.update(cx, Cloud::ensure_mcp_settings);
        let theme = Theme::get(cx).clone();
        let header = form::header(
            t!("settings-mcp-title"),
            &t!("settings-ai-access-about"),
            &theme,
        );
        let page = div().flex().flex_col().gap_8().child(header);
        let cloud = self.cloud.read(cx);
        if cloud.config().is_none() {
            let note = form::note(t!("settings-account-unavailable"), theme.muted);
            return page.child(form::card(vec![note], &theme));
        }
        if !matches!(cloud.phase(), Phase::SignedIn(_)) {
            return page.child(self.signed_out(&theme, cx));
        }
        let page = page.children(self.upgrade(&theme, cx));
        let Some(state) = cloud.mcp() else {
            return page;
        };
        let pro = cloud.can_change_mcp();
        let progress = self.progress_rows(state, pro, &theme, cx);
        let page = if progress.is_empty() {
            page
        } else {
            page.child(form::card(progress, &theme))
        };
        let Some(settings) = state.settings() else {
            return page;
        };
        let disabled = !pro || state.busy();
        let url = settings.url.clone();
        page.child(self.connect_group(&url, pro, &theme, cx))
            .child(self.access_group(disabled, &theme, cx))
    }
}

impl McpPage {
    fn signed_out(&self, theme: &Theme, cx: &mut Context<'_, McpPage>) -> Div {
        let row = form::row(
            &t!("settings-mcp-sign-in"),
            Some(&t!("settings-mcp-sign-in-about")),
            button(
                "mcp-open-account",
                t!("settings-mcp-open-account"),
                Tone::Primary,
                cx,
            )
            .on_click(cx.listener(|_, _, _, cx| cx.emit(McpPageEvent::OpenAccount))),
            theme,
        );
        form::card(vec![row], theme)
    }

    /// MCP comes with Pro; a free account sees what it'd get and the way to it.
    fn upgrade(&self, theme: &Theme, cx: &Context<'_, McpPage>) -> Option<Div> {
        let cloud = self.cloud.read(cx);
        let account = cloud.account().filter(|a| a.plan == Plan::Free)?;
        let config = cloud.config()?;
        let row = form::row(
            &t!("settings-ai-access-needs-pro"),
            Some(&t!("settings-mcp-pro-about")),
            plan::button("mcp-upgrade", account, config, cx),
            theme,
        );
        Some(form::card(vec![row], theme))
    }

    fn connect_group(
        &self,
        url: &str,
        pro: bool,
        theme: &Theme,
        cx: &mut Context<'_, McpPage>,
    ) -> Div {
        let label = if self.copied.is_some() {
            t!("settings-mcp-copied")
        } else {
            t!("settings-ai-access-copy")
        };
        let copy = button("copy-mcp-url", label, Tone::Soft, cx)
            .child(icon(
                if self.copied.is_some() {
                    "check"
                } else {
                    "copy"
                },
                gpui::px(14.),
                theme.muted,
            ))
            .flex_row_reverse();
        let copy = if pro {
            let url = url.to_owned();
            copy.on_click(cx.listener(move |this, _, _, cx| this.copy(url.clone(), cx)))
                .into_any_element()
        } else {
            copy.opacity(0.5).into_any_element()
        };
        let field = div()
            .flex()
            .items_center()
            .gap_2()
            .child(url_box(url, theme))
            .child(copy);
        let rows = vec![
            form::stacked(&t!("settings-ai-access-url"), None, field, theme),
            form::stacked(&t!("settings-mcp-how"), None, steps(theme), theme),
        ];
        form::group(&t!("settings-mcp-connect"), rows, theme)
    }

    fn access_group(&self, disabled: bool, theme: &Theme, cx: &Context<'_, McpPage>) -> Div {
        let cloud = self.cloud.read(cx);
        let settings = cloud.mcp().and_then(McpState::settings);
        let default = settings.map(|s| s.access);
        let cloud_entity = self.cloud.clone();
        let picker = segmented(
            "mcp-default",
            ACCESS.map(|a| (a, access_label(a))).to_vec(),
            default,
            disabled,
            theme,
            move |access, cx| cloud_entity.update(cx, |c, cx| c.set_mcp_access(access, cx)),
        );
        let about = default.map(default_about);
        let mut rows = vec![form::row(
            &t!("settings-ai-access-default"),
            about.as_deref(),
            picker,
            theme,
        )];
        let library = self.library.read(cx);
        for workspace in library.workspaces() {
            let id = library.workspace_sync_id(workspace.id);
            let selected = settings.map(|s| {
                id.and_then(|id| s.workspaces.get(id).copied())
                    .map_or(Choice::Default, Choice::Access)
            });
            rows.push(form::row(
                &workspace.name,
                id.is_none()
                    .then(|| t!("settings-ai-access-not-linked"))
                    .as_deref(),
                self.workspace_select(workspace.id, selected, disabled || id.is_none(), theme),
                theme,
            ));
        }
        form::group(&t!("settings-mcp-access"), rows, theme)
    }

    fn workspace_select(
        &self,
        workspace: WorkspaceId,
        selected: Option<Choice>,
        disabled: bool,
        theme: &Theme,
    ) -> AnyElement {
        let mut options: Vec<(Choice, String)> =
            vec![(Choice::Default, t!("settings-ai-access-same-as-default"))];
        options.extend(ACCESS.map(|a| (Choice::Access(a), access_label(a))));
        let cloud = self.cloud.clone();
        select(
            (
                "mcp-workspace",
                usize::try_from(workspace.0).unwrap_or_default(),
            ),
            options,
            selected,
            disabled,
            theme,
            move |choice, cx| {
                let access = match choice {
                    Choice::Default => None,
                    Choice::Access(access) => Some(access),
                };
                cloud.update(cx, |c, cx| {
                    c.set_mcp_workspace_access(workspace, access, cx);
                });
            },
        )
    }

    /// What MCP is waiting on, or what went wrong with a way to try again.
    fn progress_rows(
        &self,
        state: &McpState,
        pro: bool,
        theme: &Theme,
        cx: &Context<'_, Self>,
    ) -> Vec<AnyElement> {
        let mut rows = Vec::new();
        if state.busy() {
            let text = if state.saving() {
                t!("settings-ai-access-saving")
            } else {
                t!("settings-ai-access-loading")
            };
            rows.push(form::note(text, theme.muted));
        }
        if let Some(problem) = state.problem() {
            let again = if state.save_failed() && pro {
                t!("settings-ai-access-save-again")
            } else {
                t!("settings-ai-access-load-again")
            };
            let retry = button(
                "retry-mcp-settings",
                t!("settings-ai-access-retry"),
                Tone::Soft,
                cx,
            )
            .on_click(cx.listener(|this, _, _, cx| {
                this.cloud.update(cx, Cloud::retry_mcp_settings);
            }));
            rows.push(form::problem_note(problem, theme));
            rows.push(form::row(&again, None, retry, theme));
        }
        rows
    }
}

/// A workspace follows the default or has its own level.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Choice {
    Default,
    Access(McpAccess),
}

/// The URL in a read-only field, so it reads as something to copy.
fn url_box(url: &str, theme: &Theme) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .h(gpui::px(32.))
        .px_3()
        .flex()
        .items_center()
        .rounded(ROW_RADIUS)
        .bg(theme.soft)
        .font_family("Menlo")
        .text_xs()
        .child(div().truncate().child(url.to_owned()))
}

fn steps(theme: &Theme) -> Div {
    let step = |n: usize, text: String| {
        div()
            .flex()
            .gap_2()
            .text_sm()
            .child(
                div()
                    .w(gpui::px(16.))
                    .flex_none()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.muted)
                    .child(format!("{n}.")),
            )
            .child(div().flex_1().child(text))
    };
    div()
        .flex()
        .flex_col()
        .gap_1p5()
        .child(step(1, t!("settings-mcp-how-copy")))
        .child(step(2, t!("settings-mcp-how-add")))
        .child(step(3, t!("settings-mcp-how-sign-in")))
        .child(
            div()
                .pt_1()
                .text_xs()
                .text_color(theme.muted)
                .child(t!("settings-mcp-how-about")),
        )
}

fn access_label(access: McpAccess) -> String {
    match access {
        McpAccess::Off => t!("settings-ai-access-off"),
        McpAccess::Read => t!("settings-ai-access-read"),
        McpAccess::Full => t!("settings-ai-access-full"),
    }
}

fn default_about(access: McpAccess) -> String {
    match access {
        McpAccess::Off => t!("settings-mcp-default-off"),
        McpAccess::Read => t!("settings-mcp-default-read"),
        McpAccess::Full => t!("settings-mcp-default-full"),
    }
}
