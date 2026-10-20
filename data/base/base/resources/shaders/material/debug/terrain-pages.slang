__exported import geometry.tile;
__exported import material.debug.palette;
__exported import material.layers;
__exported import material.material;

struct TerrainPageVaryings {
    DrawVaryings draw;
    float2 faceOffset;
    nointerpolation uint tile;
};

// The page a pixel reads its color from, as the terrain debug views show it.
struct ColorPage {
    LayerRead read;
    // The level the pixel would read without a coarser page standing in.
    uint wantedLevel;
    float facing;
};

static const float pageBorderShade = 0.15;
static const float pageBorderWidthPixels = 1.5;
static const float3 noPageColor = float3(1.0);

TerrainPageVaryings terrainPageVertex(TileVertex vertex) {
    TerrainPageVaryings output;
    output.draw = vertex.draw;
    output.faceOffset = vertex.faceOffset;
    output.tile = vertex.tile;
    return output;
}

ColorPage colorPage(TerrainPageVaryings input) {
    LayerCoordinates coordinates = layerCoordinates(input.tile, input.faceOffset);
    ColorPage page;
    page.read = readLayer(coordinates, colorLayer);
    page.wantedLevel = min(uint(ceil(coordinates.level)), finestLevel(coordinates.tile, colorLayer));
    page.facing = saturate(dot(normalize(input.draw.normal), normalize(-input.draw.worldPosition)));
    return page;
}

// `color` lit by how the surface faces the camera, darkened along the edges of the page's own area.
float3 shadedPage(ColorPage page, float3 color) {
    if (!page.read.resident) {
        return noPageColor;
    }

    float2 interior = (page.read.texel + 0.5 - pageBorder) / pageInterior;
    float border = lineCoverage(interior, pageBorderWidthPixels);
    return color * lerp(ambientShade, 1.0, page.facing) * lerp(1.0, pageBorderShade, border);
}
