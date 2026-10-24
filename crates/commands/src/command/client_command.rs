use super::{
    CameraLookAtCommand, CameraModeCommand, CameraSnapCommand, CameraSpeedCommand, CameraTargetCommand,
    CameraTeleportCommand, InterfaceHideCommand, InterfacePopCommand, InterfacePushCommand, InterfaceSetCommand,
    InterfaceShowCommand, InterfaceToggleCommand,
};

pub enum ClientCommand {
    CameraMode(CameraModeCommand),
    CameraTarget(CameraTargetCommand),
    CameraTeleport(CameraTeleportCommand),
    CameraLookAt(CameraLookAtCommand),
    CameraSpeed(CameraSpeedCommand),
    CameraSnap(CameraSnapCommand),
    InterfacePush(InterfacePushCommand),
    InterfacePop(InterfacePopCommand),
    InterfaceSet(InterfaceSetCommand),
    InterfaceShow(InterfaceShowCommand),
    InterfaceHide(InterfaceHideCommand),
    InterfaceToggle(InterfaceToggleCommand),
}
