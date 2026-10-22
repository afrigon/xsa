use xsa_commands::ipc::IpcTranscript;

use super::{ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues};
use crate::config::ShellConfig;

pub struct ShellDocument {
    ipc_enabled: Option<bool>,
    ipc_input_enabled: Option<bool>,
    ipc_output_enabled: Option<bool>,
}

impl ShellDocument {
    pub const IPC_ENABLED: ConfigKey = ConfigKey::saved("shell.ipc.enabled", ConfigValueKind::Bool);
    pub const IPC_INPUT_ENABLED: ConfigKey = ConfigKey::saved("shell.ipc.input.enabled", ConfigValueKind::Bool);
    pub const IPC_OUTPUT_ENABLED: ConfigKey = ConfigKey::saved("shell.ipc.output.enabled", ConfigValueKind::Bool);

    pub fn read(values: &ConfigValues) -> ShellDocument {
        ShellDocument {
            ipc_enabled: values.bool(ShellDocument::IPC_ENABLED),
            ipc_input_enabled: values.bool(ShellDocument::IPC_INPUT_ENABLED),
            ipc_output_enabled: values.bool(ShellDocument::IPC_OUTPUT_ENABLED),
        }
    }

    pub fn into_config(self, defaults: &ShellConfig) -> ShellConfig {
        ShellConfig {
            ipc: IpcTranscript {
                enabled: self.ipc_enabled.unwrap_or(defaults.ipc.enabled),
                input: self.ipc_input_enabled.unwrap_or(defaults.ipc.input),
                output: self.ipc_output_enabled.unwrap_or(defaults.ipc.output),
            },
        }
    }

    pub fn write(config: &ShellConfig, layer: &mut ConfigLayer) {
        layer.set(ShellDocument::IPC_ENABLED.path, config.ipc.enabled.into());
        layer.set(ShellDocument::IPC_INPUT_ENABLED.path, config.ipc.input.into());
        layer.set(ShellDocument::IPC_OUTPUT_ENABLED.path, config.ipc.output.into());
    }
}
