//! The cloud sync protocol, version 1: the JSON the desktop app exchanges with the API.
//!
//! These types mirror `sdk/src/protocol.ts` in this repository, which is the
//! contract. Wire format: `snake_case` names, Unix seconds, lowercase UUID strings.

mod account;
pub mod max_chars;
mod mcp;
mod rows;
mod sync;

pub use account::{
    Account, AccountUsage, Billing, BillingStatus, ErrorResponse, ImageResponse, Limits,
    OrganizeRequest, OrganizeResponse, OrganizeScope, Organizer, Plan, ServerMessage,
};
pub use mcp::{McpAccess, McpSettings, McpSettingsResponse};
pub use rows::{
    Bookmark, BookmarkFolder, BookmarkPatch, BookmarkStatus, BookmarkTag, BookmarkText, Folder,
    FolderPatch, ListedTag, Workspace, WorkspacePatch,
};
pub use sync::{
    Change, Changed, Edit, Entity, Mutation, MutationResult, MutationStatus, Op, PullResponse,
    PushRequest, PushResponse, RejectReason, RowState,
};
use uuid::Uuid;

pub const PROTOCOL_VERSION: u32 = 1;
pub const MAX_PUSH: usize = 500;
/// The largest push request body, as UTF-8 JSON; a larger one is answered 413.
pub const MAX_PUSH_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_PULL: usize = 1000;
pub const SOCKET_PROTOCOL: &str = "knoa.v1";
pub const SOCKET_TOKEN_PREFIX: &str = "bearer.";
pub const PING: &str = "ping";
pub const PONG: &str = "pong";

/// A fresh random id, for a new workspace, folder or mutation.
pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}

/// A bookmark's id: UUID v5 of its URL's UTF-8 bytes, with its workspace's id as the
/// namespace. Two devices that import the same URL into the same workspace make the same
/// row. `None` when `workspace` isn't a UUID.
pub fn bookmark_id(workspace: &str, url: &str) -> Option<String> {
    let namespace = Uuid::parse_str(workspace).ok()?;
    Some(Uuid::new_v5(&namespace, url.as_bytes()).to_string())
}

/// The key of a `bookmark_tag` row.
pub fn tag_key(bookmark: &str, tag: &str) -> String {
    format!("{bookmark}/{tag}")
}

/// The key of a `bookmark_folder` row.
pub fn folder_link_key(bookmark: &str, folder: &str) -> String {
    format!("{bookmark}/{folder}")
}

#[cfg(test)]
mod tests {
    use super::*;

    // The web client's protocol tests pin the same value.
    #[test]
    fn a_bookmark_id_is_the_same_on_every_device() {
        let workspace = "6f1c2a3e-8b4d-4e5f-9a6b-7c8d9e0f1a2b";
        let id = bookmark_id(workspace, "https://example.com/").unwrap();
        assert_eq!(id, "cabdaf21-5e12-5864-9056-14037d3b09b8");
    }
}
