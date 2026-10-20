import terrain.output;

[shader("vertex")]
TerrainVaryings vertexMain(VertexInput input) {
    return terrainVaryings(terrainVertex(input));
}

struct EmissiveData {
    float3 luminance;
    uint padding;
};

[shader("fragment")]
FragmentOutput fragmentMain(TerrainVaryings input, uint primitive: SV_PrimitiveID) {
    MaterialSample sample;
    sample.albedo = float3(0.0);
    sample.roughness = 0.0;
    return terrainOutput(currentMaterial<EmissiveData>().luminance, sample, input, primitive);
}
