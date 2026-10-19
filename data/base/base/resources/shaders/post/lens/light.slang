import frame;
import post.lens.lens;

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
// between the inner and outer angles. The direction wraps from +180 to -180 degrees left of the light, so its
// gradient is given exactly rather than measured across neighboring pixels, which would see the jump and pick the
// coarsest mip level along that line. Closer than the inner angle, the starburst holds its innermost row.
float3 starburst(float angle, float2 angleGradient, float2 offset) {
    float logRange = log(constants.starburstExtent / constants.starburstInner);
    float stored = max(angle, constants.starburstInner);
    float distance = log(stored / constants.starburstInner) / logRange;

    if (distance > 1.0) {
        return float3(0.0);
    }

    float turn = radians(360.0);
    float2 uv = float2(atan2(offset.y, offset.x) / turn, distance);
    float2 directionGradient = float2(-offset.y, offset.x) / (dot(offset, offset) * turn);
    float2 distanceGradient = angleGradient / (stored * logRange);
    float2 uvGradientX = float2(directionGradient.x, distanceGradient.x);
    float2 uvGradientY = float2(directionGradient.y, distanceGradient.y);
    return textures[constants.starburstTexture].SampleGrad(wrappingSampler, uv, uvGradientX, uvGradientY).rgb;
}

// Light the lens spreads around each light: a glow that falls off with a power of the angle and stays flat across the
// light's own disk, and the starburst. Neither holds the light's own image, which the scene draws, so both also cover
// the disk: a body hiding part of it leaves the scattered light over the body.
[shader("fragment")]
float4 fragmentMain(LightOutput input) : SV_Target {
    ProjectedLightData light = constants.lights[input.light];
    float2 viewportSize = constants.frame->viewportSize;
    float3 toPixel = pixelRay(input.position.xy, viewportSize, constants.focalLengthPixels);
    float3 toLight = pixelRay(light.screenPosition, viewportSize, constants.focalLengthPixels);
    float angle = atan2(length(cross(toPixel, toLight)), dot(toPixel, toLight));
    float2 angleGradient = float2(ddx(angle), ddy(angle));
    float degreesFromLight = degrees(max(angle, light.angularRadius));
    float3 shape = constants.glareIntensity / pow(degreesFromLight, constants.glareFalloff);

    shape += starburst(angle, angleGradient, input.position.xy - light.screenPosition);

    float3 luminance = light.illuminance * light.visibility * light.acceptance * shape;
    return float4(luminance * constants.frame->exposure, 1.0);
}
