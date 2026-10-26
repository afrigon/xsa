import terrain.debug.height_levels;
import terrain.debug.lod;
import terrain.debug.scale;
import terrain.tile;

// Must match `TerrainDebugView` in the renderer.
enum TerrainDebugView : uint {
    None = 0,
    Lod = 1,
    Scale = 2,
    HeightLevels = 3,
}

[vk::constant_id(2)]
const uint terrainDebugView = 0;

bool showsTerrainDebugView() {
    return terrainDebugView != uint(TerrainDebugView.None);
}

float3 terrainDebugViewColor(TerrainVaryings input) {
    switch (terrainDebugView) {
    case uint(TerrainDebugView.Lod):
        return lodColor(input);
    case uint(TerrainDebugView.Scale):
        return scaleColor(input);
    default:
        return heightLevelsColor(input);
    }
}
