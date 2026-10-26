__exported import mesh.instance;

// Must match `MeshVertex` in resource-loader: its tangent frame is a quaternion rotating the tangent, bitangent and
// normal from the node's axes, negated where the texture coordinates are mirrored.
struct VertexInput {
    float3 position;
    float4 tangentFrame;
    uint instance : SV_InstanceID;
};

// What every mesh program's fragment stage reads.
struct MeshVaryings {
    DrawVaryings draw;
    nointerpolation float3 tint;
    nointerpolation float visibility;
    nointerpolation float3 groundUp;
    nointerpolation float3 groundAlbedo;
};

float3 rotated(float4 quaternion, float3 vector) {
    float3 axis = quaternion.xyz;
    float3 twice = 2.0 * cross(axis, vector);
    return vector + quaternion.w * twice + cross(axis, twice);
}

MeshVaryings meshVertex(VertexInput input) {
    MeshDrawData *draw = meshDraw();
    MeshInstanceData instance = draw.instances[input.instance];
    float4 quaternion = normalize(input.tangentFrame);
    float4 model = mul(draw.modelFromNode, float4(input.position, 1.0));
    float3 modelNormal =
        normalize(mul(draw.modelFromNode, float4(rotated(quaternion, float3(0.0, 0.0, 1.0)), 0.0)).xyz);
    float3 world = transformRows(instance.worldFromModel, model);
    float3 previousWorld = transformRows(instance.previousWorldFromModel, model);
    MeshVaryings output;
    output.draw.position = mul(constants.frame.viewProjection, float4(world, 1.0));
    output.draw.currentClip = unjittered(output.draw.position);
    output.draw.previousClip = mul(constants.frame.previousViewProjection, float4(previousWorld, 1.0));
    output.draw.worldPosition = world;
    output.draw.normal = transformRows(instance.worldFromModel, float4(modelNormal, 0.0));
    output.draw.modelNormal = modelNormal;
    output.tint = instance.tint;
    output.visibility = instance.visibility;
    output.groundUp = instance.groundUp;
    output.groundAlbedo = instance.groundAlbedo;
    return output;
}
