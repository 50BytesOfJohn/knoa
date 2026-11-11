//! Push and pull: the mutations a client sends and the changes it gets back.

use serde::{Deserialize, Serialize};

use super::rows::{
    Bookmark, BookmarkFolder, BookmarkPatch, BookmarkTag, BookmarkText, Folder, FolderPatch,
    Workspace, WorkspacePatch,
};
use crate::Origin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Entity {
    Workspace,
    Folder,
    Bookmark,
    BookmarkTag,
    BookmarkFolder,
    BookmarkText,
}

impl Entity {
    pub const ALL: [Entity; 6] = [
        Entity::Workspace,
        Entity::Folder,
        Entity::Bookmark,
        Entity::BookmarkTag,
        Entity::BookmarkFolder,
        Entity::BookmarkText,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Entity::Workspace => "workspace",
            Entity::Folder => "folder",
            Entity::Bookmark => "bookmark",
            Entity::BookmarkTag => "bookmark_tag",
            Entity::BookmarkFolder => "bookmark_folder",
            Entity::BookmarkText => "bookmark_text",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Entity> {
        Entity::ALL.into_iter().find(|entity| entity.slug() == slug)
    }
}

/// One change a client made. The server applies mutations in the order they arrive, and
/// the last one to arrive wins, column by column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mutation {
    /// Made by the client, so sending it twice is harmless.
    pub mid: String,
    pub key: String,
    pub actor: Origin,
    #[serde(flatten)]
    pub edit: Edit,
}

/// What a mutation does, to which entity. The link rows and the text are small, so they
/// are always sent whole.
#[expect(
    clippy::large_enum_variant,
    reason = "most mutations are bookmarks, so boxing them would save nothing"
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "entity", rename_all = "snake_case")]
pub enum Edit {
    Workspace(Op<WorkspacePatch>),
    Folder(Op<FolderPatch>),
    Bookmark(Op<BookmarkPatch>),
    BookmarkTag(Op<BookmarkTag>),
    BookmarkFolder(Op<BookmarkFolder>),
    BookmarkText(Op<BookmarkText>),
}

/// `put` creates the row or changes the columns in `data`; `delete` leaves a tombstone,
/// and the row's children go with it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op<T> {
    Put { data: T },
    Delete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PushRequest {
    /// At most `MAX_PUSH`.
    pub mutations: Vec<Mutation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PushResponse {
    /// One per mutation, in order. Each is final: drop the mutation whatever it says.
    pub results: Vec<MutationResult>,
    /// The library's version after this push.
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationResult {
    pub mid: String,
    pub status: MutationStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<RejectReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationStatus {
    Applied,
    Ignored,
    Rejected,
}

/// `quota`: the account is at its bookmark limit. `invalid`: the mutation makes no sense.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectReason {
    Quota,
    Invalid,
}

/// `GET /v1/sync/pull?cursor=0&limit=1000&texts=1`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PullResponse {
    /// Ordered by version. Parents can come after their children.
    pub changes: Vec<Change>,
    /// Pass as the next `cursor`.
    pub cursor: i64,
    /// There is more: pull again now.
    pub more: bool,
}

/// A row as of `version`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Change {
    pub key: String,
    pub version: i64,
    #[serde(flatten)]
    pub row: Changed,
}

#[expect(
    clippy::large_enum_variant,
    reason = "most changes are bookmarks, so boxing them would save nothing"
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "entity", rename_all = "snake_case")]
pub enum Changed {
    Workspace(RowState<Workspace>),
    Folder(RowState<Folder>),
    Bookmark(RowState<Bookmark>),
    BookmarkTag(RowState<BookmarkTag>),
    BookmarkFolder(RowState<BookmarkFolder>),
    BookmarkText(RowState<BookmarkText>),
}

impl Change {
    pub fn entity(&self) -> Entity {
        match self.row {
            Changed::Workspace(_) => Entity::Workspace,
            Changed::Folder(_) => Entity::Folder,
            Changed::Bookmark(_) => Entity::Bookmark,
            Changed::BookmarkTag(_) => Entity::BookmarkTag,
            Changed::BookmarkFolder(_) => Entity::BookmarkFolder,
            Changed::BookmarkText(_) => Entity::BookmarkText,
        }
    }
}

/// A pulled row, or its tombstone. On the wire: `{"deleted": false, "data": {..}}` or
/// `{"deleted": true}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "WireState<T>", into = "WireState<T>")]
#[serde(bound(
    serialize = "T: Serialize + Clone",
    deserialize = "T: Deserialize<'de>"
))]
pub enum RowState<T> {
    Live(T),
    Deleted,
}

#[derive(Serialize, Deserialize)]
struct WireState<T> {
    deleted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

impl<T> TryFrom<WireState<T>> for RowState<T> {
    type Error = &'static str;

    fn try_from(wire: WireState<T>) -> Result<RowState<T>, &'static str> {
        match (wire.deleted, wire.data) {
            (true, _) => Ok(RowState::Deleted),
            (false, Some(data)) => Ok(RowState::Live(data)),
            (false, None) => Err("a change that isn't a deletion needs its data"),
        }
    }
}

impl<T> From<RowState<T>> for WireState<T> {
    fn from(state: RowState<T>) -> WireState<T> {
        match state {
            RowState::Live(data) => WireState {
                deleted: false,
                data: Some(data),
            },
            RowState::Deleted => WireState {
                deleted: true,
                data: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::protocol::BookmarkStatus;

    /// Parses `wire`, checks it is `expected`, and that it serializes back to `wire`.
    fn round_trip<T>(wire: &Value, expected: &T)
    where
        T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    {
        let parsed: T = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(&parsed, expected);
        assert_eq!(&serde_json::to_value(&parsed).unwrap(), wire);
    }

    #[test]
    fn mutations_carry_only_the_columns_they_change() {
        let wire = json!({ "mutations": [
            { "mid": "m1", "entity": "bookmark", "key": "b1", "actor": "ai", "op": "put",
              "data": { "status": "organized", "summary": null } },
            { "mid": "m2", "entity": "bookmark_tag", "key": "b1/rust", "actor": "user", "op": "put",
              "data": { "bookmark_id": "b1", "tag": "rust", "origin": "user", "excluded": true } },
            { "mid": "m3", "entity": "folder", "key": "f1", "actor": "user", "op": "delete" },
        ]});
        let expected = PushRequest {
            mutations: vec![
                Mutation {
                    mid: "m1".into(),
                    key: "b1".into(),
                    actor: Origin::Ai,
                    edit: Edit::Bookmark(Op::Put {
                        data: BookmarkPatch {
                            status: Some(BookmarkStatus::Organized),
                            summary: Some(None),
                            ..BookmarkPatch::default()
                        },
                    }),
                },
                Mutation {
                    mid: "m2".into(),
                    key: "b1/rust".into(),
                    actor: Origin::User,
                    edit: Edit::BookmarkTag(Op::Put {
                        data: BookmarkTag {
                            bookmark_id: "b1".into(),
                            tag: "rust".into(),
                            origin: Origin::User,
                            excluded: true,
                        },
                    }),
                },
                Mutation {
                    mid: "m3".into(),
                    key: "f1".into(),
                    actor: Origin::User,
                    edit: Edit::Folder(Op::Delete),
                },
            ],
        };
        round_trip(&wire, &expected);
    }

    #[test]
    fn a_pull_holds_rows_and_tombstones() {
        let wire = json!({ "cursor": 42, "more": false, "changes": [
            { "entity": "folder", "key": "f1", "version": 41, "deleted": false, "data": {
                "id": "f1", "workspace_id": "w1", "name": "Reading", "description": "",
                "auto_assign": true, "color": 3, "hue": null, "icon": "book", "position": 2 } },
            { "entity": "bookmark", "key": "b1", "version": 42, "deleted": true },
        ]});
        let expected = PullResponse {
            changes: vec![
                Change {
                    key: "f1".into(),
                    version: 41,
                    row: Changed::Folder(RowState::Live(Folder {
                        id: "f1".into(),
                        workspace_id: "w1".into(),
                        name: "Reading".into(),
                        description: String::new(),
                        auto_assign: true,
                        color: 3,
                        hue: None,
                        icon: Some("book".into()),
                        position: 2,
                    })),
                },
                Change {
                    key: "b1".into(),
                    version: 42,
                    row: Changed::Bookmark(RowState::Deleted),
                },
            ],
            cursor: 42,
            more: false,
        };
        round_trip(&wire, &expected);
    }
}
