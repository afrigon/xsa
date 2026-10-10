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
    uint padding;
    float2 viewportSize;
    float2 jitter;
    float exposure;
    uint tonemapper;
    float starlightIlluminance;
    uint shadingModel;
};
