import common.scene;

// Neighboring detail levels get clearly different hues; the cycle repeats every few levels.
static const float huePerLevel = 0.17;
static const float ambientShade = 0.35;

struct LodVertexOutput {
    VertexOutput surface;
    nointerpolation int detail;
};

[shader("vertex")]
LodVertexOutput vertexMain(VertexInput input) {
    LodVertexOutput output;
    output.surface = transformVertex(input);
    output.detail = currentTile(input.instance).detail;
    return output;
}

float3 hueColor(float hue) {
    float3 channels = abs(frac(hue + float3(0.0, 2.0 / 3.0, 1.0 / 3.0)) * 6.0 - 3.0) - 1.0;
    return saturate(channels);
}

[shader("fragment")]
SurfaceOutput fragmentMain(LodVertexOutput input) {
    float3 toCamera = normalize(-input.surface.worldPosition);
    float facing = saturate(dot(normalize(input.surface.normal), toCamera));
    float3 color = hueColor(float(input.detail) * huePerLevel) * lerp(ambientShade, 1.0, facing);
    return surfaceOutput(float4(color, 1.0), input.surface);
}
