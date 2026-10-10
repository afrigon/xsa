__exported import common.frame;

struct ObjectData {
    float4x4 worldFromModel;
    float4x4 previousClipFromModel;
};

struct HapkeData {
    uint scatterTexture;
    uint surgeTexture;
    float porosity;
    float roughness;
    float blend;
    float lightBoost;
    float gammaBoost;
    uint padding;
};

struct MaterialData {
    float4 color;
    float4x4 transform;
    uint texture;
    uint normalTexture;
    uint emissiveTexture;
    float luminance;
    HapkeData hapke;
};

struct PushConstants {
    FrameData *frame;
    ObjectData *objects;
    MaterialData *materials;
    uint objectIndex;
    uint materialIndex;
};

[[vk::push_constant]]
ConstantBuffer<PushConstants> constants;

static const float pi = 3.14159265358979;
static const float uvPerNdc = 0.5;

struct VertexInput {
    float3 position;
    float3 normal;
};

struct VertexOutput {
    float4 position : SV_Position;
    float3 worldPosition;
    float3 normal;
    float3 modelNormal;
    float4 currentClip;
    float4 previousClip;
};

struct SurfaceOutput {
    float4 color : SV_Target0;
    float2 motion : SV_Target1;
};

// The jitter shifts every clip-space position by the same fraction of the viewport.
float4 unjittered(float4 clip) {
    return float4(clip.xy - constants.frame.jitter * clip.w, clip.zw);
}

// How far the surface moved across the screen since the previous frame, in texture coordinates.
float2 motionVector(float4 currentClip, float4 previousClip) {
    float2 current = currentClip.xy / currentClip.w;
    float2 previous = previousClip.xy / previousClip.w;
    return (current - previous) * uvPerNdc;
}

SurfaceOutput surfaceOutput(float4 color, VertexOutput input) {
    SurfaceOutput output;
    output.color = color;
    output.motion = motionVector(input.currentClip, input.previousClip);
    return output;
}

ObjectData currentObject() {
    return constants.objects[constants.objectIndex];
}

MaterialData currentMaterial() {
    return constants.materials[constants.materialIndex];
}

VertexOutput transformVertex(VertexInput input) {
    ObjectData object = currentObject();
    float4 world = mul(object.worldFromModel, float4(input.position, 1.0));
    VertexOutput output;
    output.position = mul(constants.frame.viewProjection, world);
    output.currentClip = unjittered(output.position);
    output.previousClip = mul(object.previousClipFromModel, float4(input.position, 1.0));
    output.worldPosition = world.xyz;
    output.normal = mul(object.worldFromModel, float4(input.normal, 0.0)).xyz;
    output.modelNormal = input.normal;
    return output;
}

float starCosine(StarData star, float3 worldPosition, float3 normal) {
    float3 toStar = normalize(star.positionAndRadius.xyz - worldPosition);
    return max(dot(normal, toStar), 0.0);
}

// Lux per color channel: the star's luminous intensity falls off with the square of the distance.
float3 starIlluminance(StarData star, float3 worldPosition) {
    float3 toStar = star.positionAndRadius.xyz - worldPosition;
    return star.intensity.rgb / dot(toStar, toStar);
}

// cd/m² reflected from the faint starlight arriving from every direction.
float3 starlightLuminance(float3 albedo) {
    return albedo / pi * constants.frame.starlightIlluminance;
}

// cd/m² reflected by a Lambertian surface of the given albedo.
float3 lambertLuminance(float3 albedo, float3 illuminance, float cosine) {
    return albedo / pi * illuminance * cosine;
}
