// Copies an image into the window pixel for pixel: a frame captured into an image of its own is shown this way.

[[vk::binding(3, 0)]]
Texture2D textures[];

// Must match CopyPushConstants in client/src/renderer/gpu_data/copy_push_constants.rs.
struct CopyConstants {
    uint sourceTexture;
};

[[vk::push_constant]]
ConstantBuffer<CopyConstants> constants;

struct FullscreenOutput {
    float4 position : SV_Position;
};

[shader("vertex")]
FullscreenOutput vertexMain(uint vertexIndex: SV_VertexID) {
    float2 clip = float2((vertexIndex << 1) & 2, vertexIndex & 2) * 2.0 - 1.0;
    FullscreenOutput output;
    output.position = float4(clip, 0.0, 1.0);
    return output;
}

[shader("fragment")]
float4 fragmentMain(FullscreenOutput input) : SV_Target {
    return textures[NonUniformResourceIndex(constants.sourceTexture)].Load(int3(int2(input.position.xy), 0));
}
