import frame;

// Must match `SurfaceTint` in the renderer.
enum SurfaceTint : uint {
    Relative = 0,
    AsIs = 1,
    Replace = 2,
}

static const float3 luminanceWeights = float3(0.2126, 0.7152, 0.0722);

// The ground's albedo once a surface of colour `surface`, averaging `average`, fades in by `fade` over the colour
// layer's `layer`, combined as the frame's tint says.
float3 tintedAlbedo(float3 layer, float3 surface, float3 average, float fade, FrameData frame) {
    float3 close;

    switch (frame.surfaceTint) {
    case uint(SurfaceTint.AsIs):
        close = layer * surface / dot(average, luminanceWeights);
        break;
    case uint(SurfaceTint.Replace):
        close = surface;
        break;
    default:
        close = layer * surface / average;
        break;
    }

    return lerp(layer, close, fade);
}
