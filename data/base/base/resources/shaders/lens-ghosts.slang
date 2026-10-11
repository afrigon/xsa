import common.frame;
import common.lens;

[[vk::binding(0, 0)]]
SamplerState clampingSampler;

[[vk::binding(3, 0)]]
Texture2D textures[];

[[vk::push_constant]]
ConstantBuffer<LensConstants> constants;

static const float2 quadCorners[6] = {
    float2(-1.0, -1.0),
    float2(1.0, -1.0),
    float2(1.0, 1.0),
    float2(-1.0, -1.0),
    float2(1.0, 1.0),
    float2(-1.0, 1.0),
};

struct GhostOutput {
    float4 position : SV_Position;
    float2 local : LOCAL;
    nointerpolation uint light : LIGHT;
    nointerpolation uint ghost : GHOST;
};

// Ghosts sit on the line from the light through the screen center: position 0 is the light, 1 the center. Each is baked
// with its local x axis along the line from the screen center toward the light.
[shader("vertex")]
GhostOutput vertexMain(uint vertexIndex: SV_VertexID, uint instanceIndex: SV_InstanceID) {
    uint lightIndex = instanceIndex / constants.ghostCount;
    uint ghostIndex = instanceIndex % constants.ghostCount;
    ProjectedLightData light = constants.lights[lightIndex];
    GhostData ghost = constants.ghosts[ghostIndex];
    float2 viewportSize = constants.frame->viewportSize;
    float2 center = lerp(light.screenPosition, viewportSize * 0.5, ghost.position);
    float halfSize = tan(ghost.size * 0.5) * constants.focalLengthPixels;
    // The quad widens along the axis so the shifted red and blue stay inside it.
    float2 corner = quadCorners[vertexIndex] * float2(1.0 + 2.0 * ghost.dispersion, 1.0);
    float2 fromCenter = light.screenPosition - viewportSize * 0.5;
    float2 axis = length(fromCenter) > 0.0 ? normalize(fromCenter) : float2(1.0, 0.0);
    float2 pixel = center + (axis * corner.x + float2(-axis.y, axis.x) * corner.y) * halfSize;
    GhostOutput output;
    output.position = float4(pixel / viewportSize * 2.0 - 1.0, 0.0, 1.0);
    output.local = corner;
    output.light = lightIndex;
    output.ghost = ghostIndex;
    return output;
}

[shader("fragment")]
float4 fragmentMain(GhostOutput input) : SV_Target {
    GhostData ghost = constants.ghosts[input.ghost];
    // The texture's border is black, so samples past its edge add nothing.
    Texture2D texture = textures[NonUniformResourceIndex(ghost.texture)];
    float2 uv = input.local * 0.5 + 0.5;
    float2 shift = float2(ghost.dispersion, 0.0);
    float red = texture.Sample(clampingSampler, uv - shift).r;
    float green = texture.Sample(clampingSampler, uv).g;
    float blue = texture.Sample(clampingSampler, uv + shift).b;
    ProjectedLightData light = constants.lights[input.light];
    float3 luminance = light.illuminance * light.visibility * light.acceptance * ghost.tint * float3(red, green, blue);
    return float4(luminance * constants.frame->exposure, 1.0);
}
