import draw;

float3 normalsColor(DrawVaryings input) {
    return normalize(input.normal) * 0.5 + 0.5;
}
