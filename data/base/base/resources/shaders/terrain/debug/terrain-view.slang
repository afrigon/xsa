import terrain.debug.lod;
import terrain.debug.page_fallback;
import terrain.debug.page_levels;
import terrain.debug.scale;
import terrain.tile;

// Must match `TerrainDebugView` in the renderer.
enum TerrainDebugView : uint {
    None = 0,
    Lod = 1,
    Scale = 2,
    PageLevels = 3,
    PageFallback = 4,
}

[vk::constant_id(2)]
const uint terrainDebugView = 0;

bool showsTerrainDebugView() {
    return terrainDebugView != uint(TerrainDebugView.None);
}

float3 terrainDebugViewColor(TileVertex input) {
    switch (terrainDebugView) {
    case uint(TerrainDebugView.Lod):
        return lodColor(input);
    case uint(TerrainDebugView.Scale):
        return scaleColor(input);
    case uint(TerrainDebugView.PageLevels):
        return pageLevelsColor(input);
    default:
        return pageFallbackColor(input);
    }
}
