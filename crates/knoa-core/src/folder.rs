//! User-made folders, nested up to [`MAX_FOLDER_DEPTH`] levels.

mod color;
mod edits;
mod emoji;
mod icon;
mod tree;

pub use color::{FolderColor, Hue, Ink};
pub use edits::FolderEdits;
pub use emoji::Emoji;
pub use icon::{FolderGlyph, FolderIcon};
pub use tree::{FolderTree, MoveError};

use crate::Origin;

/// Levels of folders a workspace can have: a top-level folder, its children and theirs.
/// `MAX_FOLDER_DEPTH` in `sdk/src/select.ts` must match.
pub const MAX_FOLDER_DEPTH: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FolderId(pub i64);

#[derive(Debug, Clone, PartialEq)]
pub struct Folder {
    pub id: FolderId,
    /// The stored parent. [`FolderTree`] decides where the folder shows when this parent
    /// is missing or would make the tree invalid.
    pub parent: Option<FolderId>,
    pub name: String,
    /// What belongs here, in the user's words. The AI matches bookmarks against it.
    pub description: String,
    /// Whether the AI may add new bookmarks to this folder.
    pub auto_assign: bool,
    pub color: FolderColor,
    /// `None` shows the folder shape.
    pub icon: Option<FolderIcon>,
    /// Bookmarks in the folder or any folder under it, each counted once.
    pub count: usize,
}

/// What a folder is created with.
#[derive(Debug, Clone, PartialEq)]
pub struct NewFolder {
    pub name: String,
    pub parent: Option<FolderId>,
    pub description: String,
    pub auto_assign: bool,
    pub color: FolderColor,
    pub icon: Option<FolderIcon>,
}

impl NewFolder {
    /// A top-level folder with the defaults: no description, auto-assign on, the first ink.
    pub fn named(name: &str) -> NewFolder {
        NewFolder {
            name: name.to_owned(),
            parent: None,
            description: String::new(),
            auto_assign: true,
            color: FolderColor::default(),
            icon: None,
        }
    }
}

/// A bookmark's membership in a folder.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FolderLink {
    pub folder: FolderId,
    pub origin: Origin,
    /// The classifier's confidence, for AI links.
    pub probability: Option<f32>,
}
