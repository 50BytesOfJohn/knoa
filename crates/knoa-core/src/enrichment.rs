//! What organizing a bookmark produced: the page's evidence and the AI's answer.

use std::fmt;

use crate::{Fields, FolderId, Image, Kind};

/// What a page fetch found, before the AI looks at it.
#[derive(Debug, Clone, Default)]
pub struct Page {
    /// The page's own title, for links added by hand that have no browser title.
    pub title: Option<String>,
    pub image: Option<Image>,
    /// Readable page text, kept locally for search.
    pub text: String,
    /// The page's own description, shown as the summary until the AI writes one.
    pub description: Option<String>,
    /// A kind and fields read directly from the page (JSON-LD, site extractors). They
    /// override the AI's values, which may be guesses.
    pub kind: Option<Kind>,
    pub fields: Fields,
    /// Everything extracted, as compact JSON for the AI prompt. Stored so organizing again
    /// later doesn't need to refetch.
    pub evidence: String,
}

/// A small copy of a bookmarked image for the AI to look at. Made fresh for each request,
/// never stored.
#[derive(Clone, PartialEq, Eq)]
pub struct AnalysisImage {
    /// Its MIME type, like `image/jpeg`.
    pub media_type: &'static str,
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Without the bytes, which would bury a log line.
impl fmt::Debug for AnalysisImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnalysisImage")
            .field("media_type", &self.media_type)
            .field("bytes", &self.bytes.len())
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

/// The AI's answer for one bookmark, already validated against the kind's fields.
#[derive(Debug, Clone)]
pub struct Enrichment {
    pub kind: Kind,
    /// A title, written only for a bookmark whose title is still its URL.
    pub title: Option<String>,
    pub summary: Option<String>,
    pub fields: Fields,
    pub extras: Vec<(String, String)>,
    pub tags: Vec<String>,
    /// Folders the classifier picked, with its probability.
    pub folders: Vec<(FolderId, f32)>,
    /// Why the bookmark doesn't fit the workspace's guideline, when it doesn't.
    pub set_aside: Option<String>,
    pub model: String,
}
