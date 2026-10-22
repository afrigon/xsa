use xsa_commands::ipc::IpcLogging;

#[derive(Clone, Debug, PartialEq)]
pub struct LoggingConfig {
    pub ipc: IpcLogging,
}
