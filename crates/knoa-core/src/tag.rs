//! Tag spelling rules, shared by user input and AI output, and a workspace's own tag list.

/// The most tags a bookmark gets from the AI, as a guard against tag spam.
pub const MAX_AI_TAGS: usize = 30;

/// Anything longer is a phrase, not a tag.
pub const MAX_TAG_CHARS: usize = 40;

/// A list's tags all go into every organizing request, so the list stays short.
pub const MAX_LISTED_TAGS: usize = 40;

/// Hints go into every organizing request too, so each is a few words.
pub const MAX_TAG_DESCRIPTION_CHARS: usize = 60;

/// Tags the user wants the AI to reach for in a workspace, so the same things get the
/// same tags. Empty by default: most libraries do fine with the AI's own words.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TagList {
    /// Normalized and sorted by name.
    pub tags: Vec<ListedTag>,
    /// The AI may add only listed tags. Turned off when the list empties.
    pub only_these: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListedTag {
    pub name: String,
    /// When the tag fits, in the user's words. Optional.
    pub description: String,
}

impl TagList {
    pub fn contains(&self, name: &str) -> bool {
        self.tags.iter().any(|tag| tag.name == name)
    }

    pub fn is_full(&self) -> bool {
        self.tags.len() >= MAX_LISTED_TAGS
    }

    /// Whether the AI is held to the list: only with a list to hold it to.
    pub fn restricts(&self) -> bool {
        self.only_these && !self.tags.is_empty()
    }
}

/// Lowercase, trimmed, words joined by hyphens: `"Film Noir "` → `"film-noir"`.
pub fn normalize_tag(tag: &str) -> String {
    tag.trim()
        .trim_start_matches('#')
        .split(|c: char| c.is_whitespace() || c == '_' || c == '-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .to_lowercase()
}
