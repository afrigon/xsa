use std::time::Instant;

use tokio::sync::oneshot;

use super::Output;

pub(crate) struct PendingWait {
    pub deadline: Instant,
    pub reply: oneshot::Sender<Output>,
}
