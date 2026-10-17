mod deletion;
mod frontier;

pub use deletion::Deletion;
pub(crate) use frontier::Frontier;

use std::collections::VecDeque;

/// Work deferred until the GPU reaches a frontier, run in the order it was queued.
///
/// Entries are queued at the frontier of the work submitted so far, which only grows, so the queue is ordered by
/// frontier and stops at the first entry the GPU has not reached. Consecutive entries at the same frontier share one
/// batch.
pub(crate) struct DeletionQueue<T> {
    batches: VecDeque<Batch<T>>,
}

struct Batch<T> {
    frontier: Frontier,
    entries: VecDeque<T>,
}

impl<T> DeletionQueue<T> {
    pub fn push(&mut self, frontier: &Frontier, entry: T) {
        if let Some(last) = self.batches.back_mut()
            && last.frontier == *frontier
        {
            last.entries.push_back(entry);
            return;
        }

        self.batches.push_back(Batch {
            frontier: frontier.clone(),
            entries: VecDeque::from([entry]),
        });
    }

    /// The oldest entry, if the GPU has reached its frontier.
    pub fn pop_reached(&mut self, completed: &Frontier) -> Option<T> {
        let batch = self.batches.front_mut()?;

        if !batch.frontier.reached_by(completed) {
            return None;
        }

        self.pop()
    }

    /// The oldest entry, whether the GPU reached it or not.
    pub fn pop(&mut self) -> Option<T> {
        let batch = self.batches.front_mut()?;
        let entry = batch.entries.pop_front();

        if batch.entries.is_empty() {
            self.batches.pop_front();
        }

        entry
    }
}

impl<T> Default for DeletionQueue<T> {
    fn default() -> Self {
        DeletionQueue {
            batches: VecDeque::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frontier(graphics: u64, presents: u64) -> Frontier {
        Frontier {
            timelines: vec![graphics],
            presents,
        }
    }

    fn reached(queue: &mut DeletionQueue<u32>, completed: &Frontier) -> Vec<u32> {
        std::iter::from_fn(|| queue.pop_reached(completed)).collect()
    }

    #[test]
    fn nothing_runs_before_its_frontier() {
        let mut queue = DeletionQueue::default();
        queue.push(&frontier(2, 0), 1);

        assert_eq!(reached(&mut queue, &frontier(1, 0)), Vec::<u32>::new());
        assert_eq!(reached(&mut queue, &frontier(2, 0)), vec![1]);
    }

    #[test]
    fn entries_run_in_the_order_they_were_queued() {
        let mut queue = DeletionQueue::default();
        queue.push(&frontier(1, 0), 1);
        queue.push(&frontier(1, 0), 2);
        queue.push(&frontier(2, 0), 3);
        queue.push(&frontier(3, 0), 4);

        assert_eq!(reached(&mut queue, &frontier(2, 0)), vec![1, 2, 3]);
        assert_eq!(reached(&mut queue, &frontier(3, 0)), vec![4]);
    }

    #[test]
    fn an_entry_waits_for_presents_too() {
        let mut queue = DeletionQueue::default();
        queue.push(&frontier(1, 1), 1);

        assert_eq!(reached(&mut queue, &frontier(5, 0)), Vec::<u32>::new());
        assert_eq!(reached(&mut queue, &frontier(5, 1)), vec![1]);
    }

    #[test]
    fn pop_ignores_frontiers() {
        let mut queue = DeletionQueue::default();
        queue.push(&frontier(1, 0), 1);
        queue.push(&frontier(9, 0), 2);

        assert_eq!(std::iter::from_fn(|| queue.pop()).collect::<Vec<_>>(), vec![1, 2]);
    }
}
