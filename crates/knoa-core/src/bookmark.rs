//! A saved link and everything Knoa knows about it.

use std::path::PathBuf;

use url::Url;

use crate::{Fields, FolderLink, Kind, WorkspaceId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BookmarkId(pub i64);

/// Where a bookmark is in the organizing pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// Imported with title and URL only; the AI hasn't been asked yet.
    New,
    /// Waiting in the organizing queue.
    Queued,
    /// Being fetched or organized right now.
    Organizing,
    Organized,
    /// Organizing gave up after retries. The bookmark keeps its title and URL.
    Failed,
}

impl Status {
    pub fn slug(self) -> &'static str {
        match self {
            Status::New => "new",
            Status::Queued => "queued",
            Status::Organizing => "organizing",
            Status::Organized => "organized",
            Status::Failed => "failed",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Status> {
        [
            Status::New,
            Status::Queued,
            Status::Organizing,
            Status::Organized,
            Status::Failed,
        ]
        .into_iter()
        .find(|s| s.slug() == slug)
    }

    pub fn is_organized(self) -> bool {
        self == Status::Organized
    }

    pub fn is_waiting(self) -> bool {
        matches!(self, Status::Queued | Status::Organizing)
    }
}

/// Who set a value. User values always win over the AI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Origin {
    Ai,
    User,
}

impl Origin {
    pub fn slug(self) -> &'static str {
        match self {
            Origin::Ai => "ai",
            Origin::User => "user",
        }
    }

    pub fn from_slug(slug: &str) -> Origin {
        if slug == "user" {
            Origin::User
        } else {
            Origin::Ai
        }
    }
}

/// A cached preview image with its pixel size, so card heights are known before decoding.
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
}

impl Image {
    pub fn aspect_ratio(&self) -> f32 {
        if self.width == 0 {
            1.0
        } else {
            self.height as f32 / self.width as f32
        }
    }
}

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub id: BookmarkId,
    pub workspace: WorkspaceId,
    pub url: String,
    pub title: String,
    pub site: String,
    /// Unix seconds, from the browser's "date added".
    pub added_at: i64,
    pub kind: Option<Kind>,
    pub summary: Option<String>,
    pub fields: Fields,
    pub extras: Vec<(String, String)>,
    pub tags: Vec<(String, Origin)>,
    pub folders: Vec<FolderLink>,
    pub up_next: bool,
    pub status: Status,
    pub image: Option<Image>,
    pub browser_folder: Option<String>,
    pub removed_from_browser: bool,
    /// Why the AI set this bookmark aside as not fitting the workspace's guideline. Set
    /// aside bookmarks only show in their own place until the user keeps them.
    pub set_aside: Option<String>,
    /// The model that organized this bookmark, for the detail panel's "why".
    pub model: Option<String>,
}

impl Bookmark {
    /// The kind's key facts as display strings ("1999", "2h 19m").
    pub fn key_facts(&self) -> Vec<String> {
        let Some(kind) = self.kind else {
            return Vec::new();
        };
        kind.definition()
            .key_facts
            .iter()
            .filter_map(|name| {
                let def = kind.field(name)?;
                let shown = def.display(self.fields.get(name)?);
                (!shown.is_empty()).then_some(shown)
            })
            .collect()
    }
}

/// What an import knows about a bookmark before Knoa has fetched anything.
#[derive(Debug, Clone, PartialEq)]
pub struct NewBookmark {
    pub url: String,
    pub title: String,
    pub added_at: i64,
    pub browser_guid: Option<String>,
    /// The folder path in the browser, `"Bookmarks Bar/Recipes"`. A hint for the AI.
    pub browser_folder: Option<String>,
}

/// The host without `www.`, which is how sites are shown and filtered.
pub fn site_of(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|u| {
            u.host_str()
                .map(|h| h.trim_start_matches("www.").to_owned())
        })
        .unwrap_or_default()
}

/// Turns what the user typed or pasted into a web URL, adding `https://` when the scheme
/// is missing. `None` when it isn't a web address.
pub fn parse_link(input: &str) -> Option<String> {
    let input = input.trim();
    if input.is_empty() || input.contains(char::is_whitespace) {
        return None;
    }
    let url = if input.contains("://") {
        Url::parse(input)
    } else {
        Url::parse(&format!("https://{input}"))
    }
    .ok()?;
    let web = matches!(url.scheme(), "http" | "https");
    // Credentials in a link are almost always a paste mistake (`mailto:` read as a user).
    let credentials = !url.username().is_empty() || url.password().is_some();
    let host = url.host_str()?;
    (web && !credentials && (host.contains('.') || host == "localhost")).then(|| url.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_links_become_web_urls() {
        assert_eq!(
            parse_link(" example.com/a ").as_deref(),
            Some("https://example.com/a")
        );
        assert_eq!(parse_link("http://x.dev").as_deref(), Some("http://x.dev/"));
        assert_eq!(parse_link("not a link"), None);
        assert_eq!(parse_link("mailto:me@x.dev"), None);
        assert_eq!(parse_link("notes"), None);
    }
}
