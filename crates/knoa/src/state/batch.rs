//! The organizing batch: the bookmarks queued since the queue was last empty, so the
//! sidebar can say "Organizing 12 of 47…" about this run rather than the whole library.

/// Tracks one run of the organizing queue from the number of bookmarks still waiting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Batch {
    /// Everything this run has taken on, done or not.
    total: usize,
    /// Still queued or being organized.
    waiting: usize,
}

/// How far a run has come.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Progress {
    /// The bookmark being worked on, counting from 1.
    pub current: usize,
    pub total: usize,
    /// Done out of the total, from 0 to 1.
    pub fraction: f32,
}

impl Batch {
    /// Takes the new waiting count. A rise adds to the run; reaching zero ends it.
    pub fn update(&mut self, waiting: usize) {
        if waiting == 0 {
            *self = Batch::default();
            return;
        }
        if waiting > self.waiting {
            self.total += waiting - self.waiting;
        }
        self.waiting = waiting;
    }

    /// `None` while nothing is waiting.
    pub fn progress(&self) -> Option<Progress> {
        if self.waiting == 0 {
            return None;
        }
        let done = self.total - self.waiting;
        Some(Progress {
            current: (done + 1).min(self.total),
            total: self.total,
            fraction: done as f32 / self.total as f32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn progress(batch: &Batch) -> (usize, usize) {
        let progress = batch.progress().expect("the batch is running");
        (progress.current, progress.total)
    }

    #[test]
    fn a_run_counts_up_and_grows_with_new_work() {
        let mut batch = Batch::default();
        batch.update(10);
        assert_eq!(progress(&batch), (1, 10));
        batch.update(7);
        assert_eq!(progress(&batch), (4, 10));
        // One more done, then two more queued.
        batch.update(6);
        batch.update(8);
        assert_eq!(progress(&batch), (5, 12));
    }

    #[test]
    fn an_emptied_queue_ends_the_run() {
        let mut batch = Batch::default();
        batch.update(3);
        batch.update(1);
        batch.update(0);
        assert_eq!(batch.progress(), None);
        batch.update(2);
        assert_eq!(progress(&batch), (1, 2));
    }
}
