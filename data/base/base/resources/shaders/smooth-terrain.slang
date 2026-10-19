import common.tile;

struct SmoothSurface : ITileSurface {
    float height(TileData tile, ObjectData object, int2 cell) {
        return 0.0;
    }
}

[shader("vertex")]
TileVertexOutput vertexMain(VertexInput input) {
    SmoothSurface surface;
    return transformTile(surface, input);
}
