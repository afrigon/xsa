use std::sync::Arc;

// Why a settled resource has no value.
pub(crate) enum Unavailable {
    Missing,
    Failed(Arc<anyhow::Error>),
}
