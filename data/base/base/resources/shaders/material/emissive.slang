import material.material;

[shader("vertex")]
DrawVaryings vertexMain<G : Geometry>(VertexInput input) {
    return G.transform(input).draw;
}

struct EmissiveData {
    float3 luminance;
    uint padding;
};

[shader("fragment")]
FragmentOutput fragmentMain(DrawVaryings input) {
    return fragmentOutput(float4(currentMaterial<EmissiveData>().luminance * constants.frame.exposure, 1.0), input);
}
