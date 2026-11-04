//! What organizing a bookmark produced: the page's evidence and the AI's answer.

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

/// The AI's answer for one bookmark, already validated against the kind's fields.
#[derive(Debug, Clone)]
pub struct Enrichment {
    pub kind: Kind,
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
