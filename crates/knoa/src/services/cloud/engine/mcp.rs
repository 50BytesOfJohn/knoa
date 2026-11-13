//! On-demand AI access requests; these never run as part of a sync round.

use gpui::Context;
use knoa_core::{
    WorkspaceId,
    protocol::{McpAccess, McpSettings, Plan},
};

use crate::services::{
    cloud::{Cloud, McpState, Phase, mcp::Change},
    network::Network,
};

enum Request {
    Load,
    Save(McpSettings),
}

impl Cloud {
    pub fn mcp(&self) -> Option<&McpState> {
        match &self.phase {
            Phase::SignedIn(connection) => Some(&connection.mcp),
            _ => None,
        }
    }

    /// Fetches when the Account page is shown, or explicitly retried.
    pub fn load_mcp_settings(&mut self, cx: &mut Context<'_, Cloud>) {
        if let Phase::SignedIn(connection) = &mut self.phase
            && connection.mcp.begin_load()
        {
            self.request_mcp(Request::Load, cx);
        }
    }

    /// Also handles signing in while the Account page is already visible.
    pub fn ensure_mcp_settings(&mut self, cx: &mut Context<'_, Cloud>) {
        if self
            .mcp()
            .is_some_and(|state| state.settings().is_none() && state.problem().is_none())
        {
            self.load_mcp_settings(cx);
        }
    }

    pub fn set_mcp_access(&mut self, access: McpAccess, cx: &mut Context<'_, Cloud>) {
        self.change_mcp(Change::Default(access), cx);
    }

    pub fn set_mcp_workspace_access(
        &mut self,
        workspace: WorkspaceId,
        access: Option<McpAccess>,
        cx: &mut Context<'_, Cloud>,
    ) {
        let Some(id) = self
            .library
            .read(cx)
            .workspace_sync_id(workspace)
            .map(str::to_owned)
        else {
            return;
        };
        self.change_mcp(Change::Workspace(id, access), cx);
    }

    /// AI access is a Pro feature; a trial counts as Pro.
    pub fn can_change_mcp(&self) -> bool {
        self.account()
            .is_some_and(|account| account.plan == Plan::Pro)
    }

    fn change_mcp(&mut self, change: Change, cx: &mut Context<'_, Cloud>) {
        if !self.can_change_mcp() {
            return;
        }
        if let Phase::SignedIn(connection) = &mut self.phase
            && let Some(settings) = connection.mcp.begin_save(change)
        {
            self.request_mcp(Request::Save(settings), cx);
        }
    }

    fn request_mcp(&mut self, request: Request, cx: &mut Context<'_, Cloud>) {
        let Phase::SignedIn(connection) = &mut self.phase else {
            return;
        };
        let api = connection.api.clone();
        let generation = self.generation.ticket();
        let task = Network::spawn(cx, async move {
            match request {
                Request::Load => api.mcp_settings().await,
                Request::Save(settings) => api.set_mcp_settings(&settings).await,
            }
        });
        connection.mcp_task = Some(cx.spawn(async move |this, cx| {
            let result = match task.await {
                Ok(result) => result.map_err(|err| err.to_string()),
                Err(err) => Err(err.to_string()),
            };
            let _ = this.update(cx, |this, cx| {
                if generation.if_current(|| ()).is_none() {
                    return;
                }
                if let Phase::SignedIn(connection) = &mut this.phase {
                    if let Err(problem) = &result {
                        tracing::warn!("AI access settings failed: {problem}");
                    }
                    connection.mcp.finish(result);
                    connection.mcp_task = None;
                    cx.notify();
                }
            });
        }));
        cx.notify();
    }
}
