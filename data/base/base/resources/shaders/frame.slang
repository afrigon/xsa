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
    uint padding[3];
    float4 whiteBalance[3];
};
