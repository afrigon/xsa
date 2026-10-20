import terrain.output;

[shader("vertex")]
TileVertex vertexMain(VertexInput input) {
    return terrainVertex(input);
}

struct EmissiveData {
    float3 luminance;
    uint padding;
};

[shader("fragment")]
FragmentOutput fragmentMain(TileVertex input, uint primitive: SV_PrimitiveID) {
    MaterialSample sample;
    sample.albedo = float3(0.0);
    sample.roughness = 0.0;
    return terrainOutput(currentMaterial<EmissiveData>().luminance, sample, input, primitive);
}
