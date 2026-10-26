mod job;
mod job_queue;
mod pool_state;

pub(crate) use job::Job;
pub(crate) use job_queue::JobQueue;
pub(crate) use pool_state::PoolState;

use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::{EntryId, LoadPriority};

// Threads that read and decode files, as many active as the worker count. Lowering it parks the extra workers once
// they finish their job; raising it wakes them, or spawns more. A read delay holds each job back, as a slow disk would.
pub(crate) struct WorkerPool {
    shared: Arc<PoolShared>,
    threads: Mutex<Vec<JoinHandle<()>>>,
}

struct PoolShared {
    state: Mutex<PoolState>,
    changed: Condvar,
}

impl WorkerPool {
    pub fn new(worker_count: usize) -> WorkerPool {
        let pool = WorkerPool {
            shared: Arc::new(PoolShared {
                state: Mutex::new(PoolState::default()),
                changed: Condvar::new(),
            }),
            threads: Mutex::new(Vec::new()),
        };
        pool.set_worker_count(worker_count);

        pool
    }

    pub fn set_worker_count(&self, count: usize) {
        let mut threads = self.threads.lock().expect("the worker threads' lock is poisoned");

        while threads.len() < count {
            let index = threads.len();
            let shared = Arc::clone(&self.shared);
            let thread = std::thread::Builder::new()
                .name(format!("resources {index}"))
                .spawn(move || shared.work(index))
                .expect("a resource worker can be spawned");
            threads.push(thread);
        }

        self.shared.state().active_workers = count;
        self.shared.changed.notify_all();
    }

    pub fn set_read_delay(&self, delay: Option<Duration>) {
        self.shared.state().read_delay = delay;
    }

    pub fn push(&self, priority: LoadPriority, job: Job) {
        self.shared.state().jobs.push(priority, job);
        self.shared.changed.notify_all();
    }

    pub fn reprioritize(&self, entry: EntryId, priority: LoadPriority) {
        self.shared.state().jobs.reprioritize(entry, priority);
    }
}

impl PoolShared {
    fn work(&self, index: usize) {
        while let Some((job, read_delay)) = self.next_job(index) {
            if let Some(delay) = read_delay {
                std::thread::sleep(delay);
            }

            (job.run)();
        }
    }

    fn next_job(&self, index: usize) -> Option<(Job, Option<Duration>)> {
        let mut state = self.state();

        loop {
            if state.shutdown {
                return None;
            }

            if index < state.active_workers
                && let Some(job) = state.jobs.pop()
            {
                return Some((job, state.read_delay));
            }

            state = self.changed.wait(state).expect("the worker pool's lock is poisoned");
        }
    }

    fn state(&self) -> MutexGuard<'_, PoolState> {
        self.state.lock().expect("the worker pool's lock is poisoned")
    }
}

// Jobs not yet taken are dropped; jobs running finish first.
impl Drop for WorkerPool {
    fn drop(&mut self) {
        self.shared.state().shutdown = true;
        self.shared.changed.notify_all();
        let threads = self.threads.get_mut().expect("the worker threads' lock is poisoned");

        for thread in threads.drain(..) {
            if thread.join().is_err() {
                tracing::error!("a resource worker panicked");
            }
        }
    }
}
