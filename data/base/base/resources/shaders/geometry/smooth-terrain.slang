import geometry.tile;

struct SmoothHeight : TileHeight {
    float height(TileData tile, ObjectData object, int2 cell) {
        return 0.0;
    }
}

struct SmoothTerrainGeometry : Geometry {
    static TileVertex transform(VertexInput input) {
        SmoothHeight heights;
        return transformTile(heights, input);
    }
}
