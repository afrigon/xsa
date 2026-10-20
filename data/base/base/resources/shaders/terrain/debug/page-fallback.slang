import terrain.debug.pages;

// How many levels coarser than wanted each pixel's color page is: none green, then yellow, orange, red from three.
static const float3 fallbackColors[4] = {
    float3(0.2, 0.9, 0.3),
    float3(1.0, 0.9, 0.2),
    float3(1.0, 0.5, 0.1),
    float3(1.0, 0.15, 0.15),
};

float3 pageFallbackColor(TerrainVaryings input) {
    ColorPage page = colorPage(input);
    uint fallback = min(page.wantedLevel - min(page.read.level, page.wantedLevel), 3u);
    return shadedPage(page, fallbackColors[fallback]);
}
