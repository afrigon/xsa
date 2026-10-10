import common.frame;

// Must match ProjectedStarData in client/src/renderer/gpu_data/projected_star_data.rs.
struct ProjectedStarData {
    float2 screenPosition;
    float pixelRadius;
    // Depth-buffer value above which a sample hides the star.
    float occlusionDepth;
    float3 illuminance;
    float angularRadius;
};

// Must match the ghost shapes in client/src/renderer/lens.rs.
static const uint ghostShapeRing = 0;
static const uint ghostShapePolygon = 1;
static const uint ghostShapeTexture = 2;

// Must match GhostData in client/src/renderer/gpu_data/ghost_data.rs.
struct GhostData {
    float position;
    float size;
    float aspectRatio;
    float intensity;
    float3 tint;
    uint shape;
    float radius;
    float width;
    float dispersion;
    float softness;
    uint streaks;
    float spots;
    uint texture;
    uint padding;
};

// Must match LensPushConstants in client/src/renderer/gpu_data/lens_push_constants.rs.
struct LensConstants {
    FrameData *frame;
    ProjectedStarData *stars;
    float *visibility;
    GhostData *ghosts;
    float glareIntensity;
    float glareFalloff;
    float focalLengthPixels;
    uint starCount;
    float spikesIntensity;
    float spikesLength;
    float spikesWidth;
    float spikesDispersion;
    uint spikesTexture;
    uint blades;
    float rotation;
    uint ghostCount;
};

// Direction of a pixel's ray, in a frame where the view axis is +Z.
float3 pixelRay(float2 pixel, float2 viewportSize, float focalLengthPixels) {
    return normalize(float3(pixel - viewportSize * 0.5, focalLengthPixels));
}
