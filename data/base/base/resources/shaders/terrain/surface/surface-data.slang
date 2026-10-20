import draw;

// Must match the renderer: `MAX_SURFACE_SCALES`, `SURFACE_CLASSES`, `TRIPLANAR_PLANES` and `SurfaceClass`.
static const int maxSurfaceScales = 6;
static const int surfaceClasses = 3;
static const int triplanarPlanes = 3;

enum SurfaceClass : uint {
    Flat = 0,
    Slope = 1,
    Steep = 2,
}

// Mirrors `SurfaceData` in the renderer. Sizes are in meters; each scale's rotation is its cosine and sine.
struct SurfaceData {
    uint colorTexture;
    uint normalTexture;
    uint displacementTexture;
    uint variantCount;
    float size;
    float texels;
    float displacementLow;
    float displacementHigh;
    float3 averageColor;
    uint scaleCount;
    float2 scaleRotations[maxSurfaceScales];
};

// Mirrors `SurfaceAnchorData`: where the camera stands within each class's texture repeat, for every scale and
// projection plane, in repeats.
struct SurfaceAnchors {
    float2 planes[surfaceClasses][maxSurfaceScales][triplanarPlanes];
};

SurfaceAnchors *surfaceAnchors() {
    return (SurfaceAnchors *)constants.surfaceAnchors;
}
