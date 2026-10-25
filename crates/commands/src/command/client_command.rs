use super::{
    CameraAction, InputCursorMoveCommand, InputCursorSetCommand, InputDownCommand, InputPressCommand,
    InputResetCommand, InputScrollCommand, InputUpCommand, InterfaceHideCommand, InterfacePopCommand,
    InterfacePushCommand, InterfaceSetCommand, InterfaceShowCommand, InterfaceToggleCommand, TimeWarpCommand,
};

use crate::value::CameraId;

pub enum ClientCommand {
    Camera { camera: CameraId, action: CameraAction },
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
