__exported import terrain.tile;
import terrain.heights;

// Must match `TileShape` in the renderer.
enum TileShape : uint {
    Smooth = 0,
    Displaced = 1,
}

[vk::constant_id(0)]
const uint tileShape = 0;

struct SmoothHeight : TileHeight {
    float height(TileData tile, ObjectData object, int2 cell) {
        return 0.0;
    }

    TerrainShape shape(TileData tile, ObjectData object, int2 cell) {
        TerrainShape shape;
        shape.height = 0.0;
        shape.curvature = 0.0;
        shape.relativeHeight = 0.0;
        return shape;
    }
}

TileVertex terrainVertex<R : TileRelief>(VertexInput input, R relief) {
    if (tileShape == uint(TileShape.Displaced)) {
        TerrainHeight heights;
        heights.pages = tilePages(constants.firstTile + input.instance);
        return transformTile(heights, relief, input);
    }

    SmoothHeight heights;
    return transformTile(heights, relief, input);
}
