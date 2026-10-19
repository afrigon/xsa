__exported import geometry.geometry;

static const float uvPerNdc = 0.5;

struct SurfaceOutput {
    float4 color : SV_Target0;
    float2 motion : SV_Target1;
};

// How far the surface moved across the screen since the previous frame, in texture coordinates.
float2 motionVector(float4 currentClip, float4 previousClip) {
    float2 current = currentClip.xy / currentClip.w;
    float2 previous = previousClip.xy / previousClip.w;
    return (current - previous) * uvPerNdc;
}

SurfaceOutput surfaceOutput(float4 color, SurfaceVaryings input) {
    SurfaceOutput output;
    output.color = color;
    output.motion = motionVector(input.currentClip, input.previousClip);
    return output;
}
