//! Knoa's domain types: workspaces, bookmarks, kinds and their fields, folders, tags,
//! search queries, AI usage, and the cloud sync protocol.
//!
//! No I/O lives here. Storage is in `knoa-store`, network work in `knoa-fetch` and `knoa-ai`.

mod bookmark;
mod enrichment;
mod field;
mod folder;
mod kind;
pub mod protocol;
mod query;
mod tag;
mod usage;
mod workspace;

pub use bookmark::{
    Bookmark, BookmarkId, Image, NewBookmark, Origin, Status, TRASH_SECONDS, parse_link, site_of,
};
pub use enrichment::{Enrichment, Page};
pub use field::{FieldDef, FieldType, Fields, format_duration};
pub use folder::{Folder, FolderColor, FolderEdits, FolderIcon, FolderId, FolderLink, Hue, Ink};
pub use kind::{Kind, KindDef};
pub use query::{Comparison, FieldFilter, Place, Query};
pub use tag::{
    ListedTag, MAX_AI_TAGS, MAX_LISTED_TAGS, MAX_TAG_CHARS, MAX_TAG_DESCRIPTION_CHARS, TagList,
    normalize_tag, tag_fits,
};
pub use usage::{ModelUsage, Usage, UsageTotals};
pub use workspace::{Screening, Workspace, WorkspaceEdits, WorkspaceId};
