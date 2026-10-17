use std::path::PathBuf;

use crate::{LoadPriority, Retention};

/// A file to load and how: who asks for it (named when it fails, and in GPU debug names), how urgently, and what
/// happens to it once unused.
#[derive(Clone, Debug)]
pub struct LoadRequest {
    pub(crate) path: PathBuf,
    pub(crate) requester: String,
    pub(crate) priority: LoadPriority,
    pub(crate) retention: Retention,
}

impl LoadRequest {
    pub fn new(path: impl Into<PathBuf>, requester: impl Into<String>) -> LoadRequest {
        LoadRequest {
            path: path.into(),
            requester: requester.into(),
            priority: LoadPriority::Normal,
            retention: Retention::Cache,
        }
    }

    pub fn priority(mut self, priority: LoadPriority) -> LoadRequest {
        self.priority = priority;
        self
    }

    pub fn retention(mut self, retention: Retention) -> LoadRequest {
        self.retention = retention;
        self
    }
}
