use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use super::scatter_request::ScatterRequest;
use super::scatter_result::ScatterResult;

// Places the instances of pages on a thread of its own, so the frame never waits for them.
pub(in crate::renderer) struct ScatterGenerator {
    requests: Option<Sender<ScatterRequest>>,
    results: Receiver<ScatterResult>,
    worker: Option<JoinHandle<()>>,
}

impl ScatterGenerator {
    pub fn new() -> anyhow::Result<ScatterGenerator> {
        let (requests, received) = mpsc::channel::<ScatterRequest>();
        let (sender, results) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("scatter placement".to_string())
            .spawn(move || {
                for request in received {
                    let result = ScatterResult {
                        scatter: request.scatter,
                        page: request.page,
                        instances: request.placement.page(request.page),
                    };

                    if sender.send(result).is_err() {
                        return;
                    }
                }
            })?;

        Ok(ScatterGenerator {
            requests: Some(requests),
            results,
            worker: Some(worker),
        })
    }

    pub fn request(&self, request: ScatterRequest) {
        if let Some(requests) = &self.requests {
            let _ = requests.send(request);
        }
    }

    pub fn results(&self) -> impl Iterator<Item = ScatterResult> + '_ {
        self.results.try_iter()
    }
}

impl Drop for ScatterGenerator {
    // Closing the requests ends the worker's loop.
    fn drop(&mut self) {
        self.requests = None;

        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
