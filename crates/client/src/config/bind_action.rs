#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BindAction {
    CameraTargetNext,
    CameraTargetPrevious,
    InterfacePauseMenu,
    InterfaceDebugOverlay,
}

impl BindAction {
    pub const ALL: [BindAction; 4] = [
        BindAction::CameraTargetNext,
        BindAction::CameraTargetPrevious,
        BindAction::InterfacePauseMenu,
        BindAction::InterfaceDebugOverlay,
    ];
}
