use tokio::sync::oneshot;

use super::Output;

pub struct CommandExecution {
    pub words: Vec<String>,
    pub reply: oneshot::Sender<Output>,
    pub styled: bool,
}
