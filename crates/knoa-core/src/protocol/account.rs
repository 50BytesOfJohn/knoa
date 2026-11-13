//! The account, the socket's messages, and the smaller endpoints.

use serde::{Deserialize, Serialize};

/// `GET /v1/me`. An `account` message on the socket says this changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub user_id: String,
    pub email: Option<String>,
    pub plan: Plan,
    pub organizer: Organizer,
    pub limits: Limits,
    pub usage: AccountUsage,
    /// Older cached accounts predate billing details; the API always provides them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billing: Option<Billing>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Plan {
    Free,
    Pro,
}

/// Who organizes the account's bookmarks: the cloud for subscribers, otherwise a Mac with
/// the user's own key, which pushes the results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Organizer {
    Cloud,
    Device,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Limits {
    pub bookmarks: u64,
    pub organized_per_month: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountUsage {
    pub bookmarks: u64,
    pub organized_this_month: u64,
}

/// Pro subscription details shared with the other clients.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Billing {
    pub status: BillingStatus,
    pub period_end: Option<i64>,
    pub cancel_at_period_end: bool,
    pub trial_available: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingStatus {
    None,
    Trialing,
    Active,
}

/// Server to client over the socket. `poke`: the library is at `version`, pull if behind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Poke { version: i64 },
    Account,
}

/// `POST /v1/organize`, subscribers only: run the cloud organizer again.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrganizeRequest {
    pub scope: OrganizeScope,
    /// Only ask the folder classifier again, over what the bookmark already has.
    pub folders_only: bool,
}

/// On the wire: `{"bookmarks": [..]}`, `{"workspace": ".."}` or `{"all": true}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganizeScope {
    Bookmarks(Vec<String>),
    Workspace(String),
    All(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrganizeResponse {
    /// Queued by this call.
    pub queued: u64,
    /// Still to be queued this month, on the server's own schedule; no need to ask again.
    #[serde(default)]
    pub pending: u64,
}

/// The answer to `PUT /v1/images/<bookmark_id>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageResponse {
    pub image_key: String,
}

/// Every error response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}
