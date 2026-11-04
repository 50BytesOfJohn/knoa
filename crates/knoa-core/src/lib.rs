//! Knoa's domain types: bookmarks, kinds and their fields, folders and search queries.
//!
//! No I/O lives here. Storage is in `knoa-store`, network work in `knoa-fetch` and `knoa-ai`.

mod bookmark;
mod enrichment;
mod field;
mod folder;
mod kind;
mod query;
mod tag;

pub use bookmark::{Bookmark, BookmarkId, Image, NewBookmark, Origin, Status, site_of};
pub use enrichment::{Enrichment, Page};
pub use field::{FieldDef, FieldType, Fields, format_duration};
pub use folder::{Folder, FolderColor, FolderId, FolderLink};
pub use kind::{Kind, KindDef};
pub use query::{Comparison, FieldFilter, Place, Query};
pub use tag::{MAX_AI_TAGS, normalize_tag};
