//! Invalidates local work when organizing moves between this Mac and the cloud.

use std::sync::{Arc, Mutex};

#[derive(Default)]
pub(super) struct Generation(Arc<Mutex<u64>>);

#[derive(Clone)]
pub(super) struct Ticket {
    generation: Arc<Mutex<u64>>,
    started: u64,
}

impl Generation {
    pub fn advance(&self) {
        *self.0.lock().expect("organizer generation poisoned") += 1;
    }

    pub fn ticket(&self) -> Ticket {
        Ticket {
            generation: self.0.clone(),
            started: *self.0.lock().expect("organizer generation poisoned"),
        }
    }
}

impl Ticket {
    // Keep the check and the write together: a switch must not slip between them.
    pub fn if_current<T>(&self, action: impl FnOnce() -> T) -> Option<T> {
        let current = self
            .generation
            .lock()
            .expect("organizer generation poisoned");
        (*current == self.started).then(action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switching_organizers_discards_old_results_even_after_switching_back() {
        let generation = Generation::default();
        let old = generation.ticket();
        assert_eq!(old.if_current(|| "saved"), Some("saved"));
        generation.advance();
        generation.advance();

        let mut saved = false;
        assert_eq!(old.if_current(|| saved = true), None);
        assert!(!saved);
        assert_eq!(generation.ticket().if_current(|| "saved"), Some("saved"));
    }
}
