#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BindAction {
    CameraDebugToggle,
    CameraTargetNext,
    CameraTargetPrevious,
    CameraReleaseMouse,
    RenderStarsToggle,
    RenderTonemapperNext,
    RenderExposureModeToggle,
    RenderBloomToggle,
    DebugShadingNext,
    DebugWireframeToggle,
    DebugShaderLit,
    DebugShaderNormals,
    DebugShaderDepth,
    DebugShaderTriangles,
    DebugShaderLighting,
}

impl BindAction {
    pub const ALL: [BindAction; 15] = [
        BindAction::CameraDebugToggle,
        BindAction::CameraTargetNext,
        BindAction::CameraTargetPrevious,
        BindAction::CameraReleaseMouse,
        BindAction::RenderStarsToggle,
        BindAction::RenderTonemapperNext,
        BindAction::RenderExposureModeToggle,
        BindAction::RenderBloomToggle,
        BindAction::DebugShadingNext,
        BindAction::DebugWireframeToggle,
        BindAction::DebugShaderLit,
        BindAction::DebugShaderNormals,
        BindAction::DebugShaderDepth,
        BindAction::DebugShaderTriangles,
        BindAction::DebugShaderLighting,
    ];
}
