//! User-made, one-level folders.

mod color;
mod icon;

pub use color::{FolderColor, Hue, Ink};
pub use icon::FolderIcon;

mod edits;

pub use edits::FolderEdits;

use crate::Origin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FolderId(pub i64);

#[derive(Debug, Clone, PartialEq)]
pub struct Folder {
    pub id: FolderId,
    pub name: String,
    /// What belongs here, in the user's words. The AI matches bookmarks against it.
    pub description: String,
    /// Whether the AI may add new bookmarks to this folder.
    pub auto_assign: bool,
    pub color: FolderColor,
    /// `None` shows the folder shape.
    pub icon: Option<FolderIcon>,
    pub count: usize,
}

/// A bookmark's membership in a folder.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FolderLink {
    pub folder: FolderId,
    pub origin: Origin,
    /// The classifier's confidence, for AI links.
    pub probability: Option<f32>,
}
