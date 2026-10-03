use super::{
    CameraLookAtCommand, CameraModeCommand, CameraTargetCommand, ConfigGetCommand, ConfigReloadCommand,
    ConfigSaveCommand, ConfigSetCommand, ConfigToggleCommand,
};

pub enum ClientCommand {
    CameraMode(CameraModeCommand),
    CameraTarget(CameraTargetCommand),
    CameraLookAt(CameraLookAtCommand),
    ConfigGet(ConfigGetCommand),
    ConfigSet(ConfigSetCommand),
    ConfigToggle(ConfigToggleCommand),
    ConfigSave(ConfigSaveCommand),
    ConfigReload(ConfigReloadCommand),
}
