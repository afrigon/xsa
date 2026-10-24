use tokio::sync::watch;
use xsa_logging::LogFilterHandle;

use super::LoggingConfig;
use crate::shell::Transcript;

// What the logging config controls in a process: the log filter and the transcripts of IPC and the startup script.
pub struct ShellOutputs {
    filter: LogFilterHandle,
    ipc: watch::Sender<Transcript>,
    script: watch::Sender<Transcript>,
}

impl ShellOutputs {
    pub fn new(filter: LogFilterHandle, config: &LoggingConfig) -> ShellOutputs {
        let outputs = ShellOutputs {
            filter,
            ipc: watch::Sender::new(config.ipc_transcript()),
            script: watch::Sender::new(config.script_transcript()),
        };
        outputs.apply(config);

        outputs
    }

    pub fn ipc_transcript(&self) -> watch::Receiver<Transcript> {
        self.ipc.subscribe()
    }

    pub fn script_transcript(&self) -> watch::Receiver<Transcript> {
        self.script.subscribe()
    }

    pub fn apply(&self, config: &LoggingConfig) {
        self.filter.apply(&config.shell_filter());
        self.ipc.send_replace(config.ipc_transcript());
        self.script.send_replace(config.script_transcript());
    }
}
