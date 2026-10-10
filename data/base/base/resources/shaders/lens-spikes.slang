import common.frame;
import common.lens;

[[vk::push_constant]]
ConstantBuffer<LensConstants> constants;

// Red diffracts farthest and blue least.
static const float3 spikesDispersionOffsets = float3(1.0, 0.0, -1.0);
// A Gaussian of full width at half maximum w is exp(-fullWidthScale * (x / w)^2).
static const float fullWidthScale = 4.0 * log(2.0);

struct SpikesOutput {
    float4 position : SV_Position;
    nointerpolation uint star : STAR;
};

[shader("vertex")]
SpikesOutput vertexMain(uint vertexIndex: SV_VertexID, uint instanceIndex: SV_InstanceID) {
    float2 clip = float2((vertexIndex << 1) & 2, vertexIndex & 2) * 2.0 - 1.0;
    SpikesOutput output;
    output.position = float4(clip, 0.0, 1.0);
    output.star = instanceIndex;
    return output;
}

// Each blade edge diffracts light into a ray perpendicular to it, on both sides: an even blade count gives one ray
// per blade, an odd one two.
[shader("fragment")]
float4 fragmentMain(SpikesOutput input) : SV_Target {
    if (constants.spikesTexture != noTexture) {
        discard;
    }

    ProjectedStarData star = constants.stars[input.star];
    float2 viewportSize = constants.frame->viewportSize;
    float3 toPixel = pixelRay(input.position.xy, viewportSize, constants.focalLengthPixels);
    float3 toStar = pixelRay(star.screenPosition, viewportSize, constants.focalLengthPixels);
    float angle = atan2(length(cross(toPixel, toStar)), dot(toPixel, toStar));
    uint rayCount = constants.blades % 2 == 0 ? constants.blades : constants.blades * 2;
    float raySpacing = radians(360.0) / rayCount;
    float2 offset = input.position.xy - star.screenPosition;
    float direction = atan2(offset.y, offset.x) - constants.rotation - radians(180.0) / constants.blades;
    float fromRay = direction - raySpacing * round(direction / raySpacing);
    float along = angle * cos(fromRay);
    float across = angle * sin(fromRay);
    float3 lengths = constants.spikesLength * (1.0 + constants.spikesDispersion * spikesDispersionOffsets);
    float3 shape = exp(-along / lengths) * exp(-fullWidthScale * pow(across / constants.spikesWidth, 2.0));
    float3 luminance = constants.spikesIntensity * star.illuminance * constants.visibility[input.star] * shape;
    return float4(luminance * constants.frame->exposure, 1.0);
}
