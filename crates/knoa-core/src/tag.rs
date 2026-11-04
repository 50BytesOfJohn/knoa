//! Tag spelling rules, shared by user input and AI output.

/// The most tags a bookmark gets from the AI, as a guard against tag spam.
pub const MAX_AI_TAGS: usize = 30;

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
