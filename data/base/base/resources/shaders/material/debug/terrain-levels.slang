import material.debug.terrain_pages;

[shader("vertex")]
TerrainPageVaryings vertexMain<G : Geometry>(VertexInput input) {
    return terrainPageVertex(G.transform(input));
}

// Each pixel takes the hue of the level of the color page it reads; page edges are dark, missing pages magenta.
[shader("fragment")]
SurfaceOutput fragmentMain(TerrainPageVaryings input) {
    ColorPage page = colorPage(input);
    float3 color = shadedPage(page, hueColor(float(page.read.level) * huePerLevel));
    return surfaceOutput(float4(color, 1.0), input.surface);
}
