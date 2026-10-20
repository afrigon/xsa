__exported import frame;

[[vk::binding(2, 0)]]
SamplerState wrappingSampler;

// Repeats both ways, for textures that tile the ground.
[[vk::binding(6, 0)]]
SamplerState tilingSampler;

[[vk::binding(3, 0)]]
Texture2D textures[];

[[vk::binding(5, 0)]]
Texture2DArray textureArrays[];

// What every vertex stage passes on, whatever places its vertices.
struct DrawVaryings {
    float4 position : SV_Position;
    float3 worldPosition;
    float3 normal;
    float3 modelNormal;
    float4 currentClip;
    float4 previousClip;
};

struct ObjectData {
    float4x4 worldFromModel;
    float4x4 previousWorldFromModel;
};

struct PushConstants {
    FrameData *frame;
    ObjectData *objects;
    void *material;
    uint objectIndex;
    uint firstTile;
    void *tiles;
    // The pages each tile reads, by tile index.
    void *tilePages;
    // Where the camera stands in the terrain's surfaces; null for a terrain without surfaces.
    void *surfaceAnchors;
};

[[vk::push_constant]]
ConstantBuffer<PushConstants> constants;

static const float pi = 3.14159265358979;

// The jitter shifts every clip-space position by the same fraction of the viewport.
float4 unjittered(float4 clip) {
    return float4(clip.xy - constants.frame.jitter * clip.w, clip.zw);
}

ObjectData currentObject() {
    return constants.objects[constants.objectIndex];
}

T currentMaterial<T>() {
    return *(T *)constants.material;
}
