import material.material;

[shader("vertex")]
SurfaceVaryings vertexMain<G : Geometry>(VertexInput input) {
    return G.transform(input).surface;
}

[shader("fragment")]
SurfaceOutput fragmentMain(SurfaceVaryings input) {
    static const float maximumDistanceExponent = 12.0;
    float distance = max(length(input.worldPosition), 1.0);
    float brightness = 1.0 - saturate(log10(distance) / maximumDistanceExponent);
    return surfaceOutput(float4(brightness.xxx, 1.0), input);
}
