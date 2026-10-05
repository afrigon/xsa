[[vk::binding(0, 0)]]
SamplerState clampingSampler;

[[vk::binding(3, 0)]]
Texture2D textures[];

[[vk::binding(4, 0)]]
[format("r11f_g11f_b10f")]
RWTexture2D<float4> storageImages[];

struct BloomConstants {
    uint sourceTexture;
    uint sourceLevel;
    uint targetImage;
    uint karisAverage;
};

[[vk::push_constant]]
ConstantBuffer<BloomConstants> constants;

float3 sampleSource(float2 uv) {
    return textures[constants.sourceTexture].SampleLevel(clampingSampler, uv, float(constants.sourceLevel)).rgb;
}
