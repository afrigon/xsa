struct FrameData
{
    float4x4 viewProjection;
    float4x4 worldFromClip;
    float4 sunPosition;
    float4 sunIntensity;
    float2 viewportSize;
    float pointQuadSize;
    float exposure;
    uint hdrTexture;
    uint tonemapping;
    float starlightIlluminance;
};
