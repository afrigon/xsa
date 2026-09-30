struct PushConstants
{
    float4x4 clipFromModel;
    float3 sunDirection;
};

[[vk::push_constant]]
ConstantBuffer<PushConstants> constants;

static const float ambient = 0.02;
static const float3 albedo = float3(0.6, 0.6, 0.6);

struct VertexInput
{
    float3 position;
    float3 normal;
};

struct VertexOutput
{
    float4 position : SV_Position;
    float3 normal;
};

[shader("vertex")]
VertexOutput vertexMain(VertexInput input)
{
    VertexOutput output;
    output.position = mul(constants.clipFromModel, float4(input.position, 1.0));
    output.normal = input.normal;
    return output;
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input) : SV_Target
{
    float diffuse = max(dot(normalize(input.normal), constants.sunDirection), 0.0);
    return float4(albedo * (diffuse + ambient), 1.0);
}
