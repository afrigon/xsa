import draw;
__exported import noise;

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
    float heightContrast;
    uint padding[3];
    float2 scaleRotations[maxSurfaceScales];
};

// Mirrors `BiomeData` in the renderer: where a biome's classes hand over, and how the ground's shape moves them.
// Angles and biases are in radians.
struct BiomeData {
    float flatToSlopeAngle;
    float flatToSlopeWidth;
    float slopeToSteepAngle;
    float slopeToSteepWidth;
    float handoverNoiseAmplitude;
    float convexBias;
    float concaveBias;
    float reliefBias;
};

// Mirrors `SurfaceAnchorData`: where the camera stands within each class's texture repeat, for every scale and
// projection plane, in repeats.
struct SurfaceAnchors {
    float2 planes[surfaceClasses][maxSurfaceScales][triplanarPlanes];
    NoiseAnchor handoverNoise;
    NoiseAnchor variantNoise[surfaceClasses];
};

SurfaceAnchors *surfaceAnchors() {
    return (SurfaceAnchors *)constants.surfaceAnchors;
}
