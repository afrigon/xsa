import material.material;

[shader("vertex")]
SurfaceVaryings vertexMain<G : Geometry>(VertexInput input) {
    return G.transform(input).surface;
}

struct EmissiveData {
    float3 luminance;
    uint padding;
};

[shader("fragment")]
SurfaceOutput fragmentMain(SurfaceVaryings input) {
    return surfaceOutput(float4(currentMaterial<EmissiveData>().luminance * constants.frame.exposure, 1.0), input);
}
