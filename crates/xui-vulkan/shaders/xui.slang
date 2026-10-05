// Must match PrimitiveInstance in src/primitive_instance.rs; std430 rounds the stride up to 64 bytes.
struct PrimitiveInstance {
    float4 bounds;
    float4 atlasRegion;
    float4 color;
    uint kind;
};

struct PushConstants {
    float2 viewportSize;
    float2 atlasSize;
};

// Must match PrimitiveInstance::GLYPH.
static const uint primitiveGlyph = 0;

[[vk::binding(0, 0)]]
Sampler2D atlas;

[[vk::binding(1, 0)]]
StructuredBuffer<PrimitiveInstance> instances;

[[vk::push_constant]]
ConstantBuffer<PushConstants> pushConstants;

struct VertexOutput {
    float4 position : SV_Position;
    float2 atlasCoordinate;
    float4 color;
    nointerpolation uint kind;
};

[shader("vertex")]
VertexOutput vertexMain(uint vertexIndex: SV_VertexID, uint instanceIndex: SV_InstanceID) {
    // Two triangles: corners 0 1 2 and 2 1 3 of the unit square.
    static const float2 corners[6] = {
        float2(0.0, 0.0),
        float2(1.0, 0.0),
        float2(0.0, 1.0),
        float2(0.0, 1.0),
        float2(1.0, 0.0),
        float2(1.0, 1.0),
    };
    float2 corner = corners[vertexIndex];
    PrimitiveInstance instance = instances[instanceIndex];
    float2 pixel = instance.bounds.xy + corner * instance.bounds.zw;

    VertexOutput output;
    output.position = float4(pixel / pushConstants.viewportSize * 2.0 - 1.0, 0.0, 1.0);
    output.atlasCoordinate = (instance.atlasRegion.xy + corner * instance.atlasRegion.zw) / pushConstants.atlasSize;
    output.color = instance.color;
    output.kind = instance.kind;
    return output;
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input) : SV_Target {
    float coverage = input.kind == primitiveGlyph ? atlas.Sample(input.atlasCoordinate).r : 1.0;
    float alpha = input.color.a * coverage;
    return float4(input.color.rgb * alpha, alpha);
}
