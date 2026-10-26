__exported import frame;

[[vk::binding(2, 0)]]
SamplerState wrappingSampler;

// Repeats along every axis, for textures that tile the ground and volumes that tile space.
[[vk::binding(6, 0)]]
SamplerState tilingSampler;

[[vk::binding(3, 0)]]
Texture2D textures[];

[[vk::binding(5, 0)]]
Texture2DArray textureArrays[];

[[vk::binding(7, 0)]]
Texture3D volumes[];

// What every vertex stage passes on, whatever places its vertices.
struct DrawVaryings {
    float4 position : SV_Position;
    float3 worldPosition;
    float3 normal;
    float3 modelNormal;
    float4 currentClip;
    float4 previousClip;
};

// What every program reads. `draw` is the draw's record, laid out by its geometry family; null for a family without
// one.
struct PushConstants {
    FrameData *frame;
    void *material;
    void *draw;
};

[[vk::push_constant]]
ConstantBuffer<PushConstants> constants;

static const float pi = 3.14159265358979;

// The jitter shifts every clip-space position by the same fraction of the viewport.
float4 unjittered(float4 clip) {
    return float4(clip.xy - constants.frame.jitter * clip.w, clip.zw);
}

T currentMaterial<T>() {
    return *(T *)constants.material;
}

// For parameters too large to copy into registers whole: fields are read where they are used.
T *currentMaterialAddress<T>() {
    return (T *)constants.material;
}
