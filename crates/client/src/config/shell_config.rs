use xsa_commands::ipc::IpcTranscript;

#[derive(Clone, Debug, PartialEq)]
pub struct ShellConfig {
    pub ipc: IpcTranscript,
}
