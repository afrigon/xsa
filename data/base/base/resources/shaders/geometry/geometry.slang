__exported import draw;

struct SurfaceVaryings {
    float4 position : SV_Position;
    float3 worldPosition;
    float3 normal;
    float3 modelNormal;
    float4 currentClip;
    float4 previousClip;
};

struct VertexInput {
    uint2 cell;
    uint instance : SV_InstanceID;
};

// Everything a tile's vertex stage computes; each material passes on the part its fragment stage reads.
struct TileVertex {
    SurfaceVaryings surface;
    float2 cell;
    int detail;
    uint gridCells;
    float heightMeters;
};

// How a draw's vertices are placed: each material's vertex stage is specialized with one per pipeline.
interface Geometry {
    static TileVertex transform(VertexInput input);
}
