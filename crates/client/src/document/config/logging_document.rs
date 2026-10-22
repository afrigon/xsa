use xsa_commands::ipc::IpcLogging;

use super::{ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues};
use crate::config::LoggingConfig;

pub struct LoggingDocument {
    ipc_enabled: Option<bool>,
    ipc_input_enabled: Option<bool>,
    ipc_output_enabled: Option<bool>,
}

impl LoggingDocument {
    pub const IPC_ENABLED: ConfigKey = ConfigKey::saved("logging.ipc.enabled", ConfigValueKind::Bool);
    pub const IPC_INPUT_ENABLED: ConfigKey = ConfigKey::saved("logging.ipc.input.enabled", ConfigValueKind::Bool);
    pub const IPC_OUTPUT_ENABLED: ConfigKey = ConfigKey::saved("logging.ipc.output.enabled", ConfigValueKind::Bool);

    pub fn read(values: &ConfigValues) -> LoggingDocument {
        LoggingDocument {
            ipc_enabled: values.bool(LoggingDocument::IPC_ENABLED),
            ipc_input_enabled: values.bool(LoggingDocument::IPC_INPUT_ENABLED),
            ipc_output_enabled: values.bool(LoggingDocument::IPC_OUTPUT_ENABLED),
        }
    }

    pub fn into_config(self, defaults: &LoggingConfig) -> LoggingConfig {
        LoggingConfig {
            ipc: IpcLogging {
                enabled: self.ipc_enabled.unwrap_or(defaults.ipc.enabled),
                input: self.ipc_input_enabled.unwrap_or(defaults.ipc.input),
                output: self.ipc_output_enabled.unwrap_or(defaults.ipc.output),
            },
        }
    }

    pub fn write(config: &LoggingConfig, layer: &mut ConfigLayer) {
        layer.set(LoggingDocument::IPC_ENABLED.path, config.ipc.enabled.into());
        layer.set(LoggingDocument::IPC_INPUT_ENABLED.path, config.ipc.input.into());
        layer.set(LoggingDocument::IPC_OUTPUT_ENABLED.path, config.ipc.output.into());
    }
}
