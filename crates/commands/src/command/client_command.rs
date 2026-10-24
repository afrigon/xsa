use super::{
    CameraLookAtCommand, CameraSnapCommand, CameraTargetCommand, CameraTeleportCommand, InputCursorMoveCommand,
    InputCursorSetCommand, InputDownCommand, InputPressCommand, InputResetCommand, InputScrollCommand, InputUpCommand,
    InterfaceHideCommand, InterfacePopCommand, InterfacePushCommand, InterfaceSetCommand, InterfaceShowCommand,
    InterfaceToggleCommand, TimeWarpCommand,
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
    InputDown(InputDownCommand),
    InputUp(InputUpCommand),
    InputPress(InputPressCommand),
    InputReset(InputResetCommand),
    InputCursorSet(InputCursorSetCommand),
    InputCursorMove(InputCursorMoveCommand),
    InputScroll(InputScrollCommand),
}
