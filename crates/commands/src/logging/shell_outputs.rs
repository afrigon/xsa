use tokio::sync::watch;
use xsa_logging::LogFilterHandle;

use super::LoggingConfig;
use crate::shell::Transcript;

// What the logging config controls in a process outside its own loop: the log filter and the transcript of IPC.
pub struct ShellOutputs {
    filter: LogFilterHandle,
    ipc: watch::Sender<Transcript>,
}

impl ShellOutputs {
    pub fn new(filter: LogFilterHandle, config: &LoggingConfig) -> ShellOutputs {
        let outputs = ShellOutputs {
            filter,
            ipc: watch::Sender::new(config.ipc_transcript()),
        };
        outputs.apply(config);

        outputs
    }

    pub fn ipc_transcript(&self) -> watch::Receiver<Transcript> {
        self.ipc.subscribe()
    }

    pub fn apply(&self, config: &LoggingConfig) {
        self.filter.apply(&config.shell_filter());
        self.ipc.send_replace(config.ipc_transcript());
    }
}
