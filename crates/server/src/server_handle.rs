use std::thread::JoinHandle;

use anyhow::anyhow;
use tokio::sync::oneshot;

// Dropping the handle stops the server too, without waiting for it.
pub struct ServerHandle {
    pub(crate) stop: oneshot::Sender<()>,
    pub(crate) thread: JoinHandle<()>,
}

impl ServerHandle {
    pub fn stop(self) -> anyhow::Result<()> {
        let _ = self.stop.send(());

        self.thread
            .join()
            .map_err(|_| anyhow!("the simulation thread panicked"))
    }
}
