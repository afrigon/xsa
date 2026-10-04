use tokio::sync::oneshot;

use super::Output;

// The answer to one invocation, sent exactly once, whenever the command finishes.
pub struct CommandReply {
    sender: oneshot::Sender<Output>,
}

impl CommandReply {
    pub fn new(sender: oneshot::Sender<Output>) -> CommandReply {
        CommandReply { sender }
    }

    pub fn send(self, result: anyhow::Result<String>) {
        let output = match result {
            Ok(text) => Output::success(text),
            Err(err) => Output::failure(format!("{err:#}")),
        };
        let _ = self.sender.send(output);
    }
}
