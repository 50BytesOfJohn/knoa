//! The synced rows, and the patches a `put` carries for the larger ones.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{Fields, Origin};

/// Defines a row and its patch: the same columns, each optional, so a `put` sends only
/// what changed. A nullable column becomes `Option<Option<_>>` in the patch, where
/// `Some(None)` sends `null` and `None` leaves the column out.
macro_rules! row_with_patch {
    (
        $(#[$meta:meta])*
        pub struct $row:ident, $patch:ident {
            $($(#[$field_meta:meta])* pub $field:ident: $ty:ty,)*
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct $row {
            $($(#[$field_meta])* pub $field: $ty,)*
        }

        #[doc = concat!("Some columns of a [`", stringify!($row), "`].")]
        #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
        pub struct $patch {
            $(
                #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "present")]
                pub $field: Option<$ty>,
            )*
        }

        impl From<$row> for $patch {
            fn from(row: $row) -> $patch {
                $patch { $($field: Some(row.$field),)* }
            }
        }
    };
}

/// A column that is in the JSON, even as `null`.
fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(deserializer).map(Some)
}

row_with_patch! {
    pub struct Workspace, WorkspacePatch {
        pub id: String,
        pub name: String,
        /// Index into the 12 inks.
        pub color: u32,
        pub position: i64,
        pub instructions: String,
        /// Whether the guideline is on.
        pub screening: bool,
        pub guideline: String,
        pub only_listed_tags: bool,
        pub tag_list: Vec<ListedTag>,
    }
}

row_with_patch! {
    pub struct Folder, FolderPatch {
        pub id: String,
        pub workspace_id: String,
        pub name: String,
        pub description: String,
        pub auto_assign: bool,
        pub color: u32,
        /// 45..=330 degrees; wins over `color`.
        pub hue: Option<u16>,
        pub icon: Option<String>,
        pub position: i64,
    }
}

row_with_patch! {
    pub struct Bookmark, BookmarkPatch {
        pub id: String,
        pub workspace_id: String,
        pub url: String,
        pub title: String,
        pub site: String,
        pub added_at: i64,
        pub up_next: bool,
        pub kept: bool,
        pub deleted_at: Option<i64>,
        pub browser_folder: Option<String>,
        pub status: BookmarkStatus,
        pub kind: Option<String>,
        pub summary: Option<String>,
        pub fields: Fields,
        pub extras: Vec<(String, String)>,
        pub set_aside: Option<String>,
        pub model: Option<String>,
        pub organized_at: Option<i64>,
        pub error: Option<String>,
        /// Path of the thumbnail under `GET /img/`.
        pub image_key: Option<String>,
        pub image_width: Option<u32>,
        pub image_height: Option<u32>,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListedTag {
    pub tag: String,
    pub description: String,
}

/// Devices keep their own queue states local: `queued` is the cloud organizer's queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BookmarkStatus {
    New,
    Queued,
    Organized,
    Failed,
}

/// Either the tag is on the bookmark, or the user removed it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookmarkTag {
    pub bookmark_id: String,
    pub tag: String,
    pub origin: Origin,
    /// The user removed it; the AI must not add it back.
    pub excluded: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookmarkFolder {
    pub bookmark_id: String,
    pub folder_id: String,
    pub origin: Origin,
    pub probability: Option<f64>,
    pub excluded: bool,
}

/// The page's readable text, for full-text search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookmarkText {
    pub bookmark_id: String,
    pub text: String,
}
