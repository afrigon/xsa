#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BindAction {
    CameraTargetNext,
    CameraTargetPrevious,
    InterfacePauseMenu,
    InterfaceDebugOverlay,
    RenderTaaToggle,
}

impl BindAction {
    pub const ALL: [BindAction; 5] = [
        BindAction::CameraTargetNext,
        BindAction::CameraTargetPrevious,
        BindAction::InterfacePauseMenu,
        BindAction::InterfaceDebugOverlay,
        BindAction::RenderTaaToggle,
    ];
}
