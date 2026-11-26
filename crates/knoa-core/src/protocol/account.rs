//! The account, the socket's messages, and the smaller endpoints.

use serde::{Deserialize, Serialize};

use crate::{Language, unknown_as_default};

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
    /// `None` until a client sets it, and from servers that predate languages.
    #[serde(default, deserialize_with = "unknown_as_default")]
    pub language: Option<Language>,
}

/// `PUT /v1/me/language`. Answers the updated [`Account`]; the socket also says `account`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageRequest {
    pub language: Language,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Plan {
    Free,
    Pro,
}

/// Who organizes the account's bookmarks: the cloud for subscribers, and for free accounts
/// while this month's free allowance has room; otherwise a Mac with the user's own key,
/// which pushes the results.
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

/// `POST /v1/organize`: run the cloud organizer again, within the month's allowance.
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrganizeResponse {
    /// Queued by this call.
    pub queued: u64,
    /// Still to be queued this month, on the server's own schedule; no need to ask again.
    #[serde(default)]
    pub pending: u64,
    /// For a `bookmarks` scope, the requested ids the server took on: queued now, already
    /// queued, kept for later, or no longer in the library. The rest were refused, which
    /// happens to a free account without room this month. Older servers leave it out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted: Option<Vec<String>>,
}

impl OrganizeResponse {
    /// Splits the `asked` ids into those the server accepted and those it refused. An
    /// answer without `accepted` comes from a server that accepted all of them.
    pub fn split(&self, asked: Vec<String>) -> (Vec<String>, Vec<String>) {
        match &self.accepted {
            Some(accepted) => asked.into_iter().partition(|id| accepted.contains(id)),
            None => (asked, Vec::new()),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn asked() -> Vec<String> {
        vec!["a".into(), "b".into(), "c".into()]
    }

    #[test]
    fn an_answer_without_accepted_accepts_everything_asked() {
        let answer: OrganizeResponse = serde_json::from_str(r#"{"queued": 3}"#).unwrap();
        assert_eq!(answer.split(asked()), (asked(), vec![]));
    }

    #[test]
    fn ids_left_out_of_accepted_were_refused() {
        let answer: OrganizeResponse =
            serde_json::from_str(r#"{"queued": 1, "pending": 0, "accepted": ["c", "a"]}"#).unwrap();
        let (accepted, refused) = answer.split(asked());
        assert_eq!(accepted, ["a", "c"]);
        assert_eq!(refused, ["b"]);
    }
}
