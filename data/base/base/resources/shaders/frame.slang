// Must match NO_TEXTURE in client/src/renderer/material.rs.
static const uint noTexture = 0xFFFFFFFF;

// Positions are relative to the camera.
struct StarData {
    float4 positionAndRadius;
    float4 intensity;
};

struct FrameData {
    float4x4 viewProjection;
    float4x4 worldFromClip;
    float4x4 previousViewProjection;
    // Without the antialiasing jitter: culling keeps the same frustum from frame to frame.
    float4x4 unjitteredViewProjection;
    StarData *stars;
    uint starCount;
    float starfieldExposure;
    float2 viewportSize;
    float2 jitter;
    float exposure;
    uint tonemapper;
    float starlightIlluminance;
    // The angle one vertex spacing of terrain covers on screen.
    float vertexSpacingAngle;
    // The angle one pixel covers at the screen's center.
    float pixelAngle;
    // Multiplies the terrain's heights as they are drawn.
    float terrainHeightScale;
    // 1 for meshes to dissolve as they fade, 0 for them to pop where their fades reach half.
    uint fadeEnabled;
    uint padding;
    float4 whiteBalance[3];
};
