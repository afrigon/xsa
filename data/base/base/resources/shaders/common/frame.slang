struct FrameData
{
    float4x4 viewProjection;
    float4x4 worldFromClip;
    float4 sunPosition;
    float4 sunIntensity;
    float2 viewportSize;
    float exposure;
    uint hdrTexture;
    uint tonemapper;
    float starlightIlluminance;
    uint bloomTexture;
    float bloomStrength;
    uint shadingModel;
};
