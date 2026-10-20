import draw;
__exported import noise;

// Must match the renderer: `SURFACE_CLASSES`, `PROJECTION_AXES`, `SURFACE_LEVELS`, `SURFACE_LEVEL_GROWTH` and
// `SurfaceClass`.
static const int surfaceClasses = 2;
static const int projectionAxes = 3;
static const int surfaceLevels = 12;
static const float surfaceLevelGrowth = 3.0;

enum SurfaceClass : uint {
    Flat = 0,
    Slope = 1,
}

// Mirrors `SurfaceData` in the renderer. Sizes are in meters.
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
    float heightContrast;
    float hexCellSize;
    uint tilingSeed;
    uint padding[2];
};

// Mirrors `BiomeData` in the renderer: where a biome's classes hand over, and how the ground's shape moves them.
// Angles and biases are in radians.
struct BiomeData {
    float transitionAngle;
    float transitionWidth;
    float handoverNoiseAmplitude;
    float convexBias;
    float concaveBias;
    float reliefBias;
    uint padding[2];
};

// Mirrors `HexAnchor`: where the camera stands on a lattice of hexagonal cells, skewed.
struct HexAnchor {
    int2 cell;
    float2 fraction;
};

// Mirrors `SurfaceAnchorData`: where the camera stands on each class's hexagonal cells, for every projection plane and
// level.
struct SurfaceAnchors {
    HexAnchor planes[surfaceClasses][projectionAxes][surfaceLevels];
    NoiseAnchor handoverNoise;
    NoiseAnchor variantNoise[surfaceClasses];
    uint noiseVolume;
    uint padding[3];
};

SurfaceAnchors *surfaceAnchors() {
    return (SurfaceAnchors *)constants.surfaceAnchors;
}
