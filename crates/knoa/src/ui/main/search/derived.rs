//! What the search's parts work out from the library, kept between frames.

use knoa_core::Place;

use crate::state::search::Search;

/// A value worked out for one search and place. It is kept until either changes or it is
/// forgotten, which happens when the library loads new results and on a language switch.
#[derive(Default)]
pub(super) struct Derived<T> {
    key: Option<(Search, Place)>,
    value: T,
}

impl<T> Derived<T> {
    /// The value for `search` at `place`, built only if it isn't kept already.
    pub fn get(&mut self, search: &Search, place: Place, build: impl FnOnce() -> T) -> &T {
        let kept = self
            .key
            .as_ref()
            .is_some_and(|(s, p)| s == search && *p == place);
        if !kept {
            self.value = build();
            self.key = Some((search.clone(), place));
        }
        &self.value
    }

    pub fn forget(&mut self) {
        self.key = None;
    }
}
