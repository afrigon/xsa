import material.debug.palette;
import material.material;

struct LodVaryings {
    SurfaceVaryings surface;
    float2 cell;
    nointerpolation int detail;
    nointerpolation uint gridCells;
};

[shader("vertex")]
LodVaryings vertexMain<G : Geometry>(VertexInput input) {
    TileVertex tile = G.transform(input);
    LodVaryings output;
    output.surface = tile.surface;
    output.cell = tile.cell;
    output.detail = tile.detail;
    output.gridCells = tile.gridCells;
    return output;
}

static const float cellLineShade = 0.55;
static const float tileEdgeShade = 0.1;
static const float lineWidthPixels = 1.0;
static const float tileEdgeWidthPixels = 2.0;

[shader("fragment")]
SurfaceOutput fragmentMain(LodVaryings input) {
    float3 toCamera = normalize(-input.surface.worldPosition);
    float facing = saturate(dot(normalize(input.surface.normal), toCamera));
    float3 color = hueColor(float(input.detail) * huePerLevel) * lerp(ambientShade, 1.0, facing);
    float cellLine = lineCoverage(input.cell, lineWidthPixels);
    float tileEdge = lineCoverage(input.cell / float(input.gridCells), tileEdgeWidthPixels);
    color *= lerp(1.0, cellLineShade, cellLine);
    color *= lerp(1.0, tileEdgeShade, tileEdge);
    return surfaceOutput(float4(color, 1.0), input.surface);
}
