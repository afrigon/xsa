import draw;

static const float maximumDistanceExponent = 12.0;

float3 depthColor(DrawVaryings input) {
    float distance = max(length(input.worldPosition), 1.0);
    float brightness = 1.0 - saturate(log10(distance) / maximumDistanceExponent);
    return brightness.xxx;
}
