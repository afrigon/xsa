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
