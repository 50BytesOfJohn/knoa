//! The Account page's AI access URL, default, and workspace exceptions.

use gpui::{
    ClipboardItem, Context, Div, IntoElement as _, SharedString, StatefulInteractiveElement as _,
};
use gpui_component::radio::{Radio, RadioGroup};
use knoa_core::{
    WorkspaceId,
    protocol::{McpAccess, Plan},
};

use super::{AccountPage, form, plan};
use crate::{
    services::cloud::{Cloud, McpState},
    ui::{
        components::{Tone, button},
        theme::Theme,
    },
};

const ABOUT: &str = "Let AI apps like Claude and ChatGPT read and manage your bookmarks.";
const ACCESS: [(McpAccess, &str); 3] = [
    (McpAccess::Off, "Off"),
    (McpAccess::Read, "Read only"),
    (McpAccess::Full, "Full access"),
];

impl AccountPage {
    pub(super) fn ai_access_group(&self, theme: &Theme, cx: &mut Context<'_, Self>) -> Div {
        let cloud = self.cloud.read(cx);
        let state = cloud.mcp();
        let settings = state.and_then(McpState::settings);
        let pro = cloud.can_change_mcp();
        let busy = state.is_some_and(McpState::busy);
        let disabled = !pro || busy || settings.is_none();
        let mut rows = vec![form::note(ABOUT, theme.text)];
        if let Some(settings) = settings {
            let url = settings.url.clone();
            let copy = if pro {
                button("copy-mcp-url", "Copy", Tone::Soft, cx)
                    .on_click(move |_, _, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(url.clone()));
                    })
                    .into_any_element()
            } else {
                gpui::Empty.into_any_element()
            };
            rows.push(form::row("MCP URL", Some(&settings.url), copy, theme));
        }
        rows.push(form::stacked(
            "Default access",
            Some("Applies to workspaces set to Same as default."),
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
                    .then_some("Available once this workspace is linked to the cloud."),
                choices(Some(workspace.id), selected, disabled || id.is_none(), cx),
                theme,
            ));
        }
        if let Some(state) = state {
            if busy {
                rows.push(form::note(
                    if state.saving() {
                        "Saving…"
                    } else {
                        "Loading AI access…"
                    },
                    theme.muted,
                ));
            }
            if let Some(problem) = state.problem() {
                rows.push(form::problem_note(problem, theme));
                rows.push(form::row(
                    if state.save_failed() && pro {
                        "Save AI access again"
                    } else {
                        "Load AI access again"
                    },
                    None,
                    button("retry-mcp-settings", "Retry", Tone::Soft, cx).on_click(cx.listener(
                        |this, _, _, cx| this.cloud.update(cx, Cloud::retry_mcp_settings),
                    )),
                    theme,
                ));
            }
        }
        if let Some(account) = cloud.account()
            && account.plan == Plan::Free
            && let Some(config) = cloud.config()
        {
            rows.push(form::row(
                "AI access needs Knoa Pro",
                None,
                plan::button("mcp-upgrade", account, config, cx),
                theme,
            ));
        }
        form::group("AI access", rows, theme)
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
        options.push((Choice::Default, "Same as default"));
    }
    options.extend(ACCESS.map(|(access, label)| (Choice::Access(access), label)));
    let index =
        selected.and_then(|selected| options.iter().position(|(access, _)| *access == selected));
    RadioGroup::horizontal(id)
        .children(
            options
                .iter()
                .enumerate()
                .map(|(index, (_, label))| Radio::new(index).label(*label)),
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
