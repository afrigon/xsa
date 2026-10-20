// Must match `SurfaceDebugView` in the renderer.
enum SurfaceDebugView : uint {
    None = 0,
    Classes = 1,
    Scales = 2,
    Planes = 3,
    Fade = 4,
}

[vk::constant_id(4)]
const uint surfaceDebugView = 0;

bool showsSurfaceDebugView() {
    return surfaceDebugView != uint(SurfaceDebugView.None);
}
