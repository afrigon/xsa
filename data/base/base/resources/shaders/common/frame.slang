struct FrameData {
    float4x4 viewProjection;
    float4x4 worldFromClip;
    float4x4 previousViewProjection;
    float4 sunPosition;
    float4 sunIntensity;
    float2 viewportSize;
    float2 jitter;
    float exposure;
    uint sceneColorTexture;
    uint tonemapper;
    float starlightIlluminance;
    uint bloomTexture;
    float bloomStrength;
    uint shadingModel;
};
