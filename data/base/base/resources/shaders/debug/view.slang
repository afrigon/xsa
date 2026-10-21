__exported import fragment;
import debug.depth;
import debug.lighting;
import debug.motion;
import debug.normals;
import debug.triangles;

// Must match `DebugView` in the renderer.
enum DebugView : uint {
    None = 0,
    Normals = 1,
    Depth = 2,
    Motion = 3,
    Lighting = 4,
    Triangles = 5,
    Albedo = 6,
    Roughness = 7,
}

[vk::constant_id(1)]
const uint debugView = 0;

// Roughness is drawn from flat (black) to this many radians of θ̄ (white).
static const float roughestShown = pi / 4.0;

bool showsDebugView() {
    return debugView != uint(DebugView.None);
}

float3 debugViewColor(DrawVaryings input, MaterialSample sample, uint primitive) {
    switch (debugView) {
    case uint(DebugView.Normals):
        return normalsColor(input);
    case uint(DebugView.Depth):
        return depthColor(input);
    case uint(DebugView.Motion):
        return motionColor(input);
    case uint(DebugView.Lighting):
        return lightingColor(input);
    case uint(DebugView.Triangles):
        return trianglesColor(primitive);
    case uint(DebugView.Albedo):
        return sample.albedo;
    default:
        return saturate(sample.roughness / roughestShown).xxx;
    }
}
