import draw;
__exported import noise;

// Must match the renderer: `SURFACE_CLASSES`, `TRIPLANAR_PLANES` and `SurfaceClass`.
static const int surfaceClasses = 3;
static const int triplanarPlanes = 3;

enum SurfaceClass : uint {
    Flat = 0,
    Slope = 1,
    Steep = 2,
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
    float flatToSlopeAngle;
    float flatToSlopeWidth;
    float slopeToSteepAngle;
    float slopeToSteepWidth;
    float handoverNoiseAmplitude;
    float convexBias;
    float concaveBias;
    float reliefBias;
};

// Mirrors `HexAnchor`: where the camera stands on a lattice of hexagonal cells, skewed.
struct HexAnchor {
    int2 cell;
    float2 fraction;
};

// Mirrors `SurfaceAnchorData`: where the camera stands on each class's hexagonal cells, for every projection plane.
struct SurfaceAnchors {
    HexAnchor planes[surfaceClasses][triplanarPlanes];
    NoiseAnchor handoverNoise;
    NoiseAnchor variantNoise[surfaceClasses];
    uint noiseVolume;
    uint padding[3];
};

SurfaceAnchors *surfaceAnchors() {
    return (SurfaceAnchors *)constants.surfaceAnchors;
}
