//! The longest value the server takes per text column, in characters (Unicode code
//! points), as `MAX_CHARS` in `sdk/src/protocol.ts`. A longer one gets the whole mutation
//! rejected as `invalid`. Only the columns the app lets someone type into are here.

pub mod workspace {
    pub const NAME: usize = 200;
    pub const INSTRUCTIONS: usize = 10_000;
    pub const GUIDELINE: usize = 10_000;
}

pub mod folder {
    pub const NAME: usize = 200;
    pub const DESCRIPTION: usize = 2_000;
}

/// Whether `text` is at most `max` characters, counted as the server counts them.
pub fn fits(text: &str, max: usize) -> bool {
    text.chars().count() <= max
}
