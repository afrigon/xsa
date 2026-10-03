use super::{
    CameraLookAtCommand, CameraModeCommand, CameraSnapCommand, CameraTargetCommand, ConfigGetCommand,
    ConfigReloadCommand, ConfigSaveCommand, ConfigSetCommand, ConfigToggleCommand,
};

pub enum ClientCommand {
    CameraMode(CameraModeCommand),
    CameraTarget(CameraTargetCommand),
    CameraLookAt(CameraLookAtCommand),
    CameraSnap(CameraSnapCommand),
    ConfigGet(ConfigGetCommand),
    ConfigSet(ConfigSetCommand),
    ConfigToggle(ConfigToggleCommand),
    ConfigSave(ConfigSaveCommand),
    ConfigReload(ConfigReloadCommand),
}
