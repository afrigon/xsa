import geometry.tile;

struct SmoothSurface : TileSurface {
    float height(TileData tile, ObjectData object, int2 cell) {
        return 0.0;
    }
}

struct SmoothTerrainGeometry : Geometry {
    static TileVertex transform(VertexInput input) {
        SmoothSurface surface;
        return transformTile(surface, input);
    }
}
