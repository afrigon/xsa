use super::{
    CameraLookAtCommand, CameraSnapCommand, CameraTargetCommand, CameraTeleportCommand, InterfaceHideCommand,
    InterfacePopCommand, InterfacePushCommand, InterfaceSetCommand, InterfaceShowCommand, InterfaceToggleCommand,
    TimeWarpCommand,
};

pub enum ClientCommand {
    CameraTarget(CameraTargetCommand),
    CameraTeleport(CameraTeleportCommand),
    CameraLookAt(CameraLookAtCommand),
    CameraSnap(CameraSnapCommand),
    InterfacePush(InterfacePushCommand),
    InterfacePop(InterfacePopCommand),
    InterfaceSet(InterfaceSetCommand),
    InterfaceShow(InterfaceShowCommand),
    InterfaceHide(InterfaceHideCommand),
    InterfaceToggle(InterfaceToggleCommand),
    TimeWarp(TimeWarpCommand),
}
