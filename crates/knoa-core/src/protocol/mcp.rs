//! Settings for AI clients connected over MCP.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// What AI clients connected over MCP may do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpAccess {
    Off,
    Read,
    Full,
}

/// The complete replacement body for `PUT /v1/mcp/settings`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpSettings {
    /// Applies to every workspace without an entry in `workspaces`.
    pub access: McpAccess,
    /// Exceptions by workspace id. A workspace not listed uses `access`.
    pub workspaces: BTreeMap<String, McpAccess>,
}

/// `GET` and `PUT /v1/mcp/settings`, including the address to paste into an AI client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpSettingsResponse {
    pub access: McpAccess,
    pub workspaces: BTreeMap<String, McpAccess>,
    pub url: String,
}
