//! Workspaces: separate libraries, each fed by its own browser profiles and organized by
//! its own instructions, folders and tag list.

use crate::TagList;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorkspaceId(pub i64);

#[derive(Debug, Clone, PartialEq)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    /// An index into the theme's palette, like folder colors.
    pub color: u8,
    /// Browser profiles synced into this workspace, as `helium:Default`. A profile may feed
    /// several workspaces; each keeps its own copy of the bookmarks.
    pub browsers: Vec<String>,
    /// The browser profile its bookmarks open in, as `helium:Default`. `None` uses the
    /// system's default browser.
    pub opens_in: Option<String>,
    /// How the user wants things organized here, in their own words.
    pub instructions: String,
    pub screening: Screening,
    pub tag_list: TagList,
}

/// An optional rule for which synced bookmarks this workspace keeps. Off by default, so
/// everything the browser has lands in the workspace.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Screening {
    pub enabled: bool,
    /// What belongs in this workspace, in the user's words. Kept while screening is off so
    /// turning it back on doesn't lose it.
    pub guideline: String,
}

impl Screening {
    /// The guideline the AI should apply, if screening is on and says anything.
    pub fn active_guideline(&self) -> Option<&str> {
        let guideline = self.guideline.trim();
        (self.enabled && !guideline.is_empty()).then_some(guideline)
    }
}
