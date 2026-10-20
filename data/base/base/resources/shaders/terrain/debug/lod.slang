import debug.palette;
import terrain.tile;

static const float cellLineShade = 0.55;
static const float tileEdgeShade = 0.1;
static const float lineWidthPixels = 1.0;
static const float tileEdgeWidthPixels = 2.0;

float3 lodColor(TerrainVaryings input) {
    float3 toCamera = normalize(-input.draw.worldPosition);
    float facing = saturate(dot(normalize(input.draw.normal), toCamera));
    float3 color = hueColor(float(input.detail) * huePerLevel) * lerp(ambientShade, 1.0, facing);
    float cellLine = lineCoverage(input.cell, lineWidthPixels);
    float tileEdge = lineCoverage(input.cell / float(input.gridCells), tileEdgeWidthPixels);
    color *= lerp(1.0, cellLineShade, cellLine);
    color *= lerp(1.0, tileEdgeShade, tileEdge);
    return color;
}
