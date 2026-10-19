__exported import frame;

[[vk::binding(2, 0)]]
SamplerState wrappingSampler;

[[vk::binding(3, 0)]]
Texture2D textures[];

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
    // Read only by the terrain's vertex stage.
    void *terrain;
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
