import common.terrain;

[shader("vertex")]
TileVertexOutput vertexMain(VertexInput input) {
    TerrainSurface surface;
    return transformTile(surface, input);
}
