use std::collections::VecDeque;

use crate::worker_pool::Job;
use crate::{EntryId, LoadPriority};

// Jobs waiting for a worker, a queue per priority, run most urgent first and in the order they were queued within a
// priority.
#[derive(Default)]
pub(crate) struct JobQueue {
    levels: [VecDeque<Job>; LoadPriority::ALL.len()],
}

impl JobQueue {
    pub fn push(&mut self, priority: LoadPriority, job: Job) {
        self.levels[priority.index()].push_back(job);
    }

    pub fn pop(&mut self) -> Option<Job> {
        self.levels.iter_mut().rev().find_map(VecDeque::pop_front)
    }

    // Moves a waiting job to the back of `priority`'s queue; nothing happens to one a worker took.
    pub fn reprioritize(&mut self, entry: EntryId, priority: LoadPriority) {
        let found = self.levels.iter_mut().find_map(|level| {
            let position = level.iter().position(|job| job.entry == entry)?;
            level.remove(position)
        });

        if let Some(job) = found {
            self.push(priority, job);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(value: u64) -> Job {
        Job {
            entry: EntryId { value },
            run: Box::new(|| {}),
        }
    }

    fn order(queue: &mut JobQueue) -> Vec<u64> {
        std::iter::from_fn(|| queue.pop()).map(|job| job.entry.value).collect()
    }

    #[test]
    fn urgent_jobs_run_first_then_in_queued_order() {
        let mut queue = JobQueue::default();
        queue.push(LoadPriority::Background, job(0));
        queue.push(LoadPriority::Normal, job(1));
        queue.push(LoadPriority::Critical, job(2));
        queue.push(LoadPriority::Normal, job(3));

        assert_eq!(order(&mut queue), vec![2, 1, 3, 0]);
    }

    #[test]
    fn a_raised_job_runs_sooner() {
        let mut queue = JobQueue::default();
        queue.push(LoadPriority::Normal, job(0));
        queue.push(LoadPriority::Background, job(1));
        queue.reprioritize(EntryId { value: 1 }, LoadPriority::High);

        assert_eq!(order(&mut queue), vec![1, 0]);
    }
}
