import common.frame;
import common.lens;

[[vk::binding(2, 0)]]
SamplerState wrappingSampler;

[[vk::binding(3, 0)]]
Texture2D textures[];

[[vk::push_constant]]
ConstantBuffer<LensConstants> constants;

struct LightOutput {
    float4 position : SV_Position;
    nointerpolation uint light : LIGHT;
};

[shader("vertex")]
LightOutput vertexMain(uint vertexIndex: SV_VertexID, uint instanceIndex: SV_InstanceID) {
    float2 clip = float2((vertexIndex << 1) & 2, vertexIndex & 2) * 2.0 - 1.0;
    LightOutput output;
    output.position = float4(clip, 0.0, 1.0);
    output.light = instanceIndex;
    return output;
}

// The starburst is stored log-polar: u is the direction around the light, v the angle from it on a log scale
// between the inner and outer angles.
float3 starburst(float angle, float2 offset) {
    float distance = log(angle / constants.starburstInner) / log(constants.starburstExtent / constants.starburstInner);

    if (distance < 0.0 || distance > 1.0) {
        return float3(0.0);
    }

    float2 uv = float2(atan2(offset.y, offset.x) / radians(360.0), distance);
    return textures[constants.starburstTexture].Sample(wrappingSampler, uv).rgb;
}

// Light the lens spreads around each light: a glow that falls off with a power of the angle and stays flat across the
// light's own disk, and the starburst outside that disk, which the scene already draws.
[shader("fragment")]
float4 fragmentMain(LightOutput input) : SV_Target {
    ProjectedLightData light = constants.lights[input.light];
    float2 viewportSize = constants.frame->viewportSize;
    float3 toPixel = pixelRay(input.position.xy, viewportSize, constants.focalLengthPixels);
    float3 toLight = pixelRay(light.screenPosition, viewportSize, constants.focalLengthPixels);
    float angle = atan2(length(cross(toPixel, toLight)), dot(toPixel, toLight));
    float degreesFromLight = degrees(max(angle, light.angularRadius));
    float3 shape = constants.glareIntensity / pow(degreesFromLight, constants.glareFalloff);

    if (angle >= light.angularRadius) {
        shape += starburst(angle, input.position.xy - light.screenPosition);
    }

    float3 luminance = light.illuminance * light.visibility * shape;
    return float4(luminance * constants.frame->exposure, 1.0);
}
