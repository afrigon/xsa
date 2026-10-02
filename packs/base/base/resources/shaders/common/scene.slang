struct FrameData
{
    float4x4 viewProjection;
    float4x4 worldFromClip;
    float4 sunPosition;
    float2 viewportSize;
    float pointQuadSize;
};

struct ObjectData
{
    float4x4 worldFromModel;
    float pointDiameter;
    float pointIntensity;
    float2 padding;
};

struct MaterialData
{
    float4 color;
    float4x4 transform;
    uint texture;
    uint normalTexture;
    uint emissiveTexture;
    uint padding;
};

struct PushConstants
{
    FrameData* frame;
    ObjectData* objects;
    MaterialData* materials;
    uint objectIndex;
    uint materialIndex;
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
    float3 modelNormal;
};

ObjectData currentObject()
{
    return constants.objects[constants.objectIndex];
}

MaterialData currentMaterial()
{
    return constants.materials[constants.materialIndex];
}

VertexOutput transformVertex(VertexInput input)
{
    ObjectData object = currentObject();
    float4 world = mul(object.worldFromModel, float4(input.position, 1.0));
    VertexOutput output;
    output.position = mul(constants.frame.viewProjection, world);
    output.worldPosition = world.xyz;
    output.normal = mul(object.worldFromModel, float4(input.normal, 0.0)).xyz;
    output.modelNormal = input.normal;
    return output;
}

float sunlight(VertexOutput input)
{
    float3 toSun = normalize(constants.frame.sunPosition.xyz - input.worldPosition);
    return max(dot(normalize(input.normal), toSun), 0.0) + ambient;
}
