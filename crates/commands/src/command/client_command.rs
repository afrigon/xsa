use super::{
    CameraLookAtCommand, CameraModeCommand, CameraSnapCommand, CameraTargetCommand, ConfigGetCommand,
    ConfigReloadCommand, ConfigSaveCommand, ConfigSetCommand, ConfigToggleCommand, InterfaceHideCommand,
    InterfacePopCommand, InterfacePushCommand, InterfaceSetCommand, InterfaceShowCommand, InterfaceToggleCommand,
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
    InterfacePush(InterfacePushCommand),
    InterfacePop(InterfacePopCommand),
    InterfaceSet(InterfaceSetCommand),
    InterfaceShow(InterfaceShowCommand),
    InterfaceHide(InterfaceHideCommand),
    InterfaceToggle(InterfaceToggleCommand),
}
