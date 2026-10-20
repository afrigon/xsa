import material.material;

[shader("vertex")]
DrawVaryings vertexMain<G : Geometry>(VertexInput input) {
    return G.transform(input).draw;
}

[shader("fragment")]
FragmentOutput fragmentMain(DrawVaryings input) {
    return fragmentOutput(float4(normalize(input.normal) * 0.5 + 0.5, 1.0), input);
}
