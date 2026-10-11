import common.frame;
import common.lens;

[[vk::binding(0, 0)]]
SamplerState clampingSampler;

[[vk::binding(3, 0)]]
Texture2D textures[];

[[vk::push_constant]]
ConstantBuffer<LensConstants> constants;

struct GlareOutput {
    float4 position : SV_Position;
    nointerpolation uint light : LIGHT;
};

[shader("vertex")]
GlareOutput vertexMain(uint vertexIndex: SV_VertexID, uint instanceIndex: SV_InstanceID) {
    float2 clip = float2((vertexIndex << 1) & 2, vertexIndex & 2) * 2.0 - 1.0;
    GlareOutput output;
    output.position = float4(clip, 0.0, 1.0);
    output.light = instanceIndex;
    return output;
}

// Veiling glare falls off with a power of the angle from the light, and stays flat across the light's own disk.
[shader("fragment")]
float4 fragmentMain(GlareOutput input) : SV_Target {
    ProjectedLightData light = constants.lights[input.light];
    float3 toPixel = pixelRay(input.position.xy, constants.frame->viewportSize, constants.focalLengthPixels);
    float3 toLight = pixelRay(light.screenPosition, constants.frame->viewportSize, constants.focalLengthPixels);
    float angle = atan2(length(cross(toPixel, toLight)), dot(toPixel, toLight));
    float degreesFromLight = degrees(max(angle, light.angularRadius));
    float3 luminance =
        constants.glareIntensity * light.illuminance * light.visibility / pow(degreesFromLight, constants.glareFalloff);
    return float4(luminance * constants.frame->exposure, 1.0);
}
