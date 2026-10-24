use super::{
    CameraLookAtCommand, CameraSnapCommand, CameraSpeedCommand, CameraTargetCommand, CameraTeleportCommand,
    InterfaceHideCommand, InterfacePopCommand, InterfacePushCommand, InterfaceSetCommand, InterfaceShowCommand,
    InterfaceToggleCommand,
};

pub enum ClientCommand {
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
