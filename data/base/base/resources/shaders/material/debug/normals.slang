import material.material;

[shader("vertex")]
SurfaceVaryings vertexMain<G : Geometry>(VertexInput input) {
    return G.transform(input).surface;
}

[shader("fragment")]
SurfaceOutput fragmentMain(SurfaceVaryings input) {
    return surfaceOutput(float4(normalize(input.normal) * 0.5 + 0.5, 1.0), input);
}
