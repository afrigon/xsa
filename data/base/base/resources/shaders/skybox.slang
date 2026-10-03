import common.scene;

[[vk::binding(0, 0)]]
SamplerState linearSampler;

[[vk::binding(1, 0)]]
TextureCube cubeTextures[];

struct SkyOutput
{
    float4 position : SV_Position;
    float2 clip;
};

[shader("vertex")]
SkyOutput vertexMain(uint vertexIndex : SV_VertexID)
{
    float2 clip = float2((vertexIndex << 1) & 2, vertexIndex & 2) * 2.0 - 1.0;
    SkyOutput output;
    output.position = float4(clip, 0.0, 1.0);
    output.clip = clip;
    return output;
}

[shader("fragment")]
float4 fragmentMain(SkyOutput input) : SV_Target
{
    float4 world = mul(constants.frame.worldFromClip, float4(input.clip, 1.0, 1.0));
    float3 direction = normalize(world.xyz / world.w);
    MaterialData material = currentMaterial();
    float3 cubeDirection = mul(material.transform, float4(direction, 0.0)).xyz;
    float3 sky = cubeTextures[material.texture].Sample(linearSampler, cubeDirection).rgb;
    return float4(sky * material.luminance * constants.frame.exposure, 1.0);
}
