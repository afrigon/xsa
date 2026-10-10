import common.frame;
import common.lens;

[[vk::binding(0, 0)]]
SamplerState clampingSampler;

[[vk::binding(3, 0)]]
Texture2D textures[];

struct LensConstants {
    FrameData *frame;
    ProjectedStarData *stars;
    float *visibility;
    GhostData *ghosts;
    float glareIntensity;
    float glareFalloff;
    float focalLengthPixels;
    uint starCount;
    float spikesIntensity;
    float spikesLength;
    float spikesWidth;
    float spikesDispersion;
    uint spikesTexture;
    uint blades;
    float rotation;
    uint ghostCount;
};

[[vk::push_constant]]
ConstantBuffer<LensConstants> constants;

struct GlareOutput {
    float4 position : SV_Position;
    nointerpolation uint star : STAR;
};

// Direction of a pixel's ray, in a frame where the view axis is +Z.
float3 pixelRay(float2 pixel) {
    return normalize(float3(pixel - constants.frame->viewportSize * 0.5, constants.focalLengthPixels));
}

[shader("vertex")]
GlareOutput vertexMain(uint vertexIndex: SV_VertexID, uint instanceIndex: SV_InstanceID) {
    float2 clip = float2((vertexIndex << 1) & 2, vertexIndex & 2) * 2.0 - 1.0;
    GlareOutput output;
    output.position = float4(clip, 0.0, 1.0);
    output.star = instanceIndex;
    return output;
}

// Veiling glare falls off with a power of the angle from the star, and stays flat across the star's own disk.
[shader("fragment")]
float4 fragmentMain(GlareOutput input) : SV_Target {
    ProjectedStarData star = constants.stars[input.star];
    float3 toPixel = pixelRay(input.position.xy);
    float3 toStar = pixelRay(star.screenPosition);
    float angle = atan2(length(cross(toPixel, toStar)), dot(toPixel, toStar));
    float degreesFromStar = degrees(max(angle, star.angularRadius));
    float3 luminance = constants.glareIntensity * star.illuminance * constants.visibility[input.star] /
                       pow(degreesFromStar, constants.glareFalloff);
    return float4(luminance * constants.frame->exposure, 1.0);
}
