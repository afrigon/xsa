__exported import draw;

// One drawn instance of a mesh, mirroring `MeshInstanceData` in the renderer: the rows of its transform from the
// model to the world, relative to this frame's camera, and of the previous frame's, relative to that frame's camera;
// its tint on the material's base colour; how visible it is, faded out by dithering; and the ground it stands on, which
// reflects light onto it: the ground's up direction in the world and its albedo (black for an instance on no ground).
struct MeshInstanceData {
    float4 worldFromModel[3];
    float4 previousWorldFromModel[3];
    float3 tint;
    float visibility;
    float3 groundUp;
    uint padding;
    float3 groundAlbedo;
    uint groundPadding;
};

// A mesh draw's record, mirroring `MeshDrawData` in the renderer: the drawn node's place in the model, and the first of
// the draw's instances.
struct MeshDrawData {
    float4x4 modelFromNode;
    MeshInstanceData *instances;
    uint2 padding;
};

MeshDrawData *meshDraw() {
    return (MeshDrawData *)constants.draw;
}

float3 transformRows(float4 rows[3], float4 point) {
    return float3(dot(rows[0], point), dot(rows[1], point), dot(rows[2], point));
}
