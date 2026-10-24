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

    pub fn category(self) -> &'static str {
        match self {
            BindAction::CameraTargetNext | BindAction::CameraTargetPrevious => "camera",
            BindAction::InterfacePauseMenu | BindAction::InterfaceDebugOverlay => "interface",
            BindAction::TimeFaster | BindAction::TimeSlower | BindAction::TimeRealTime => "time",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            BindAction::CameraTargetNext => "target-next",
            BindAction::CameraTargetPrevious => "target-previous",
            BindAction::InterfacePauseMenu => "pause-menu",
            BindAction::InterfaceDebugOverlay => "debug-overlay",
            BindAction::TimeFaster => "faster",
            BindAction::TimeSlower => "slower",
            BindAction::TimeRealTime => "real-time",
        }
    }
}
