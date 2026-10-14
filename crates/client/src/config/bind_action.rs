#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BindAction {
    CameraTargetNext,
    CameraTargetPrevious,
    InterfacePauseMenu,
    InterfaceDebugOverlay,
    TimeFaster,
    TimeSlower,
    TimeRealTime,
}

impl BindAction {
    pub const ALL: [BindAction; 7] = [
        BindAction::CameraTargetNext,
        BindAction::CameraTargetPrevious,
        BindAction::InterfacePauseMenu,
        BindAction::InterfaceDebugOverlay,
        BindAction::TimeFaster,
        BindAction::TimeSlower,
        BindAction::TimeRealTime,
    ];
}
