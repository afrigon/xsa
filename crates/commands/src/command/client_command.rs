use super::{
    CameraLookAtCommand, CameraModeCommand, CameraSnapCommand, CameraSpeedCommand, CameraTargetCommand,
    CameraTeleportCommand, ConfigGetCommand, ConfigReloadCommand, ConfigSaveCommand, ConfigSetCommand,
    ConfigToggleCommand, InterfaceHideCommand, InterfacePopCommand, InterfacePushCommand, InterfaceSetCommand,
    InterfaceShowCommand, InterfaceToggleCommand,
};

pub enum ClientCommand {
    CameraMode(CameraModeCommand),
    CameraTarget(CameraTargetCommand),
    CameraTeleport(CameraTeleportCommand),
    CameraLookAt(CameraLookAtCommand),
    CameraSpeed(CameraSpeedCommand),
    CameraSnap(CameraSnapCommand),
    ConfigGet(ConfigGetCommand),
    ConfigSet(ConfigSetCommand),
    ConfigToggle(ConfigToggleCommand),
    ConfigSave(ConfigSaveCommand),
    ConfigReload(ConfigReloadCommand),
    InterfacePush(InterfacePushCommand),
    InterfacePop(InterfacePopCommand),
    InterfaceSet(InterfaceSetCommand),
    InterfaceShow(InterfaceShowCommand),
    InterfaceHide(InterfaceHideCommand),
    InterfaceToggle(InterfaceToggleCommand),
}
