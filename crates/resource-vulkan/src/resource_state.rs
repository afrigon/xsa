use std::sync::Arc;

/// Where a resource's load stands.
#[derive(Clone, Debug)]
pub enum ResourceState {
    Loading,
    Ready,
    /// The file does not exist.
    Missing,
    /// The file exists but could not be loaded.
    Failed(Arc<anyhow::Error>),
}
