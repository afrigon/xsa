struct FrameData
{
    float4x4 viewProjection;
    float4 sunPosition;
    float2 viewportSize;
    float pointQuadSize;
};

struct ObjectData
{
    float4x4 worldFromModel;
    float4 color;
    float pointDiameter;
    float pointIntensity;
    float2 padding;
};

struct PushConstants
{
    FrameData* frame;
    ObjectData* objects;
    uint objectIndex;
};

[[vk::push_constant]]
ConstantBuffer<PushConstants> constants;

static const float ambient = 0.02;

struct VertexInput
{
    float3 position;
    float3 normal;
};

struct VertexOutput
{
    float4 position : SV_Position;
    float3 worldPosition;
    float3 normal;
};

ObjectData currentObject()
{
    return constants.objects[constants.objectIndex];
}

VertexOutput transformVertex(VertexInput input)
{
    ObjectData object = currentObject();
    float4 world = mul(object.worldFromModel, float4(input.position, 1.0));
    VertexOutput output;
    output.position = mul(constants.frame.viewProjection, world);
    output.worldPosition = world.xyz;
    output.normal = mul(object.worldFromModel, float4(input.normal, 0.0)).xyz;
    return output;
}

float sunlight(VertexOutput input)
{
    float3 toSun = normalize(constants.frame.sunPosition.xyz - input.worldPosition);
    return max(dot(normalize(input.normal), toSun), 0.0) + ambient;
}
