import frame;

// Must match `SurfaceTint` in the renderer.
enum SurfaceTint : uint {
    Relative = 0,
    AsIs = 1,
    Replace = 2,
}

static const float3 luminanceWeights = float3(0.2126, 0.7152, 0.0722);

// A surface's colour as the frame's tint combines it with the colour layer: relative to its own average colour, at an
// average brightness of 1, or as it is.
float3 tintSpace(float3 color, float3 average, FrameData frame) {
    switch (frame.surfaceTint) {
    case uint(SurfaceTint.AsIs):
        return color / dot(average, luminanceWeights);
    case uint(SurfaceTint.Replace):
        return color;
    default:
        return color / average;
    }
}

// The ground's albedo once a surface, in its tint space, fades in by `fade` over the colour layer's `layer`.
float3 tintedAlbedo(float3 layer, float3 surface, float fade, FrameData frame) {
    if (frame.surfaceTint == uint(SurfaceTint.Replace)) {
        return lerp(layer, surface, fade);
    }

    return layer * lerp(float3(1.0), surface, fade);
}
