use tokio::sync::oneshot;

use super::Completions;

pub struct CommandCompletion {
    pub line: String,
    pub cursor: usize,
    pub reply: oneshot::Sender<Completions>,
}
