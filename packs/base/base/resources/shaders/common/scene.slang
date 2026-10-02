__exported import common.frame;

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
    float luminance;
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

static const float pi = 3.14159265358979;

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

float sunCosine(float3 worldPosition, float3 normal)
{
    float3 toSun = normalize(constants.frame.sunPosition.xyz - worldPosition);
    return max(dot(normal, toSun), 0.0);
}

// Lux per color channel: the star's luminous intensity falls off with the square of the distance.
float3 sunIlluminance(float3 worldPosition)
{
    float3 toSun = constants.frame.sunPosition.xyz - worldPosition;
    return constants.frame.sunIntensity.rgb / dot(toSun, toSun);
}

// cd/m² reflected by a Lambertian surface of the given albedo, including the faint starlight from every direction.
float3 lambertLuminance(float3 albedo, float3 illuminance, float cosine)
{
    return albedo / pi * (illuminance * cosine + constants.frame.starlightIlluminance);
}
