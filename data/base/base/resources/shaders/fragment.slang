__exported import draw;

static const float uvPerNdc = 0.5;

// What a program's material is at a pixel, as the debug views read it. Roughness is Hapke's θ̄ in radians, 0 for
// models without one.
struct MaterialSample {
    float3 albedo;
    float roughness;
};

struct FragmentOutput {
    float4 color : SV_Target0;
    float2 motion : SV_Target1;
};

// How far the surface moved across the screen since the previous frame, in texture coordinates.
float2 motionVector(float4 currentClip, float4 previousClip) {
    float2 current = currentClip.xy / currentClip.w;
    float2 previous = previousClip.xy / previousClip.w;
    return (current - previous) * uvPerNdc;
}

FragmentOutput fragmentOutput(float4 color, DrawVaryings input) {
    FragmentOutput output;
    output.color = color;
    output.motion = motionVector(input.currentClip, input.previousClip);
    return output;
}
