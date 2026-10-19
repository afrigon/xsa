import material.debug.terrain_pages;

[shader("vertex")]
TerrainPageVaryings vertexMain<G : Geometry>(VertexInput input) {
    return terrainPageVertex(G.transform(input));
}

// How many levels coarser than wanted each pixel's color page is: none green, then yellow, orange, red from three.
static const float3 fallbackColors[4] = {
    float3(0.2, 0.9, 0.3),
    float3(1.0, 0.9, 0.2),
    float3(1.0, 0.5, 0.1),
    float3(1.0, 0.15, 0.15),
};

[shader("fragment")]
SurfaceOutput fragmentMain(TerrainPageVaryings input) {
    ColorPage page = colorPage(input);
    uint fallback = min(page.wantedLevel - min(page.read.level, page.wantedLevel), 3u);
    float3 color = shadedPage(page, fallbackColors[fallback]);
    return surfaceOutput(float4(color, 1.0), input.surface);
}
