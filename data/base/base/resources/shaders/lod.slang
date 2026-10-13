import common.scene;

// Neighboring detail levels get clearly different hues; the cycle repeats every few levels.
static const float huePerLevel = 0.17;
static const float ambientShade = 0.35;
static const float cellLineShade = 0.55;
static const float tileEdgeShade = 0.1;
static const float lineWidthPixels = 1.0;
static const float tileEdgeWidthPixels = 2.0;

struct LodVertexOutput {
    VertexOutput surface;
    float2 cell;
    nointerpolation int detail;
    nointerpolation uint gridCells;
};

[shader("vertex")]
LodVertexOutput vertexMain(VertexInput input) {
    TileData tile = currentTile(input.instance);
    LodVertexOutput output;
    output.surface = transformVertex(input);
    output.cell = float2(input.cell);
    output.detail = tile.detail;
    output.gridCells = tile.gridCells;
    return output;
}

float3 hueColor(float hue) {
    float3 channels = abs(frac(hue + float3(0.0, 2.0 / 3.0, 1.0 / 3.0)) * 6.0 - 3.0) - 1.0;
    return saturate(channels);
}

// 1 on a line through whole values of `coordinates`, fading to 0 over `widthPixels` on screen.
float lineCoverage(float2 coordinates, float widthPixels) {
    float2 distanceToLine = abs(coordinates - round(coordinates)) / fwidth(coordinates);
    return 1.0 - saturate(min(distanceToLine.x, distanceToLine.y) / widthPixels);
}

[shader("fragment")]
SurfaceOutput fragmentMain(LodVertexOutput input) {
    float3 toCamera = normalize(-input.surface.worldPosition);
    float facing = saturate(dot(normalize(input.surface.normal), toCamera));
    float3 color = hueColor(float(input.detail) * huePerLevel) * lerp(ambientShade, 1.0, facing);
    float cellLine = lineCoverage(input.cell, lineWidthPixels);
    float tileEdge = lineCoverage(input.cell / float(input.gridCells), tileEdgeWidthPixels);
    color *= lerp(1.0, cellLineShade, cellLine);
    color *= lerp(1.0, tileEdgeShade, tileEdge);
    return surfaceOutput(float4(color, 1.0), 0.0, input.surface);
}
