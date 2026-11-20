//! The Account page's AI access URL, default, and workspace exceptions.

use gpui::{
    AnyElement, ClipboardItem, Context, Div, IntoElement as _, SharedString,
    StatefulInteractiveElement as _,
};
use gpui_component::radio::{Radio, RadioGroup};
use knoa_core::{
    WorkspaceId,
    protocol::{McpAccess, Plan},
};

use super::{AccountPage, form, plan};
use crate::{
    i18n::t,
    services::cloud::{Cloud, McpState},
    ui::{
        components::{Tone, button},
        theme::Theme,
    },
};

const ACCESS: [McpAccess; 3] = [McpAccess::Off, McpAccess::Read, McpAccess::Full];

impl AccountPage {
    pub(super) fn ai_access_group(&self, theme: &Theme, cx: &mut Context<'_, Self>) -> Div {
        let cloud = self.cloud.read(cx);
        let state = cloud.mcp();
        let settings = state.and_then(McpState::settings);
        let pro = cloud.can_change_mcp();
        let busy = state.is_some_and(McpState::busy);
        let disabled = !pro || busy || settings.is_none();
        let mut rows = vec![form::note(t!("settings-ai-access-about"), theme.text)];
        if let Some(settings) = settings {
            let url = settings.url.clone();
            let copy = if pro {
                button(
                    "copy-mcp-url",
                    t!("settings-ai-access-copy"),
                    Tone::Soft,
                    cx,
                )
                .on_click(move |_, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(url.clone()));
                })
                .into_any_element()
            } else {
                gpui::Empty.into_any_element()
            };
            rows.push(form::row(
                &t!("settings-ai-access-url"),
                Some(&settings.url),
                copy,
                theme,
            ));
        }
        rows.push(form::stacked(
            &t!("settings-ai-access-default"),
            Some(&t!("settings-ai-access-default-about")),
            choices(
                None,
                settings.map(|s| Choice::Access(s.access)),
                disabled,
                cx,
            ),
            theme,
        ));
        let library = self.library.read(cx);
        for workspace in library.workspaces() {
            let id = library.workspace_sync_id(workspace.id);
            let selected = settings.map(|s| {
                id.and_then(|id| s.workspaces.get(id).copied())
                    .map_or(Choice::Default, Choice::Access)
            });
            rows.push(form::stacked(
                &workspace.name,
                id.is_none()
                    .then(|| t!("settings-ai-access-not-linked"))
                    .as_deref(),
                choices(Some(workspace.id), selected, disabled || id.is_none(), cx),
                theme,
            ));
        }
        if let Some(state) = state {
            rows.extend(self.progress_rows(state, pro, theme, cx));
        }
        if let Some(account) = cloud.account()
            && account.plan == Plan::Free
            && let Some(config) = cloud.config()
        {
            rows.push(form::row(
                &t!("settings-ai-access-needs-pro"),
                None,
                plan::button("mcp-upgrade", account, config, cx),
                theme,
            ));
        }
        form::group(&t!("settings-ai-access-title"), rows, theme)
    }

    /// What AI access is waiting on, or what went wrong with a way to try again.
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Choice {
    Default,
    Access(McpAccess),
}

fn choices(
    workspace: Option<WorkspaceId>,
    selected: Option<Choice>,
    disabled: bool,
    cx: &Context<'_, AccountPage>,
) -> RadioGroup {
    let id = SharedString::from(match workspace {
        Some(id) => format!("mcp-workspace-{}", id.0),
        None => "mcp-default".to_owned(),
    });
    let mut options = Vec::new();
    if workspace.is_some() {
        options.push((Choice::Default, t!("settings-ai-access-same-as-default")));
    }
    options.extend(ACCESS.map(|access| (Choice::Access(access), access_label(access))));
    let index =
        selected.and_then(|selected| options.iter().position(|(access, _)| *access == selected));
    RadioGroup::horizontal(id)
        .children(
            options
                .iter()
                .enumerate()
                .map(|(index, (_, label))| Radio::new(index).label(label.clone())),
        )
        .selected_index(index)
        .disabled(disabled)
        .on_change(cx.listener(move |this, index: &usize, _, cx| {
            if disabled {
                return;
            }
            if let Some((access, _)) = options.get(*index) {
                this.cloud
                    .update(cx, |cloud, cx| match (workspace, access) {
                        (Some(workspace), Choice::Default) => {
                            cloud.set_mcp_workspace_access(workspace, None, cx);
                        }
                        (Some(workspace), Choice::Access(access)) => {
                            cloud.set_mcp_workspace_access(workspace, Some(*access), cx);
                        }
                        (None, Choice::Access(access)) => cloud.set_mcp_access(*access, cx),
                        (None, Choice::Default) => {}
                    });
            }
        }))
}

fn access_label(access: McpAccess) -> String {
    match access {
        McpAccess::Off => t!("settings-ai-access-off"),
        McpAccess::Read => t!("settings-ai-access-read"),
        McpAccess::Full => t!("settings-ai-access-full"),
    }
}
