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
    nointerpolation uint star : STAR;
    nointerpolation uint ghost : GHOST;
};

// Ghosts sit on the line from the star through the screen center: position 0 is the star, 1 the center.
[shader("vertex")]
GhostOutput vertexMain(uint vertexIndex: SV_VertexID, uint instanceIndex: SV_InstanceID) {
    uint starIndex = instanceIndex / constants.ghostCount;
    uint ghostIndex = instanceIndex % constants.ghostCount;
    ProjectedStarData star = constants.stars[starIndex];
    GhostData ghost = constants.ghosts[ghostIndex];
    float2 viewportSize = constants.frame->viewportSize;
    float2 center = lerp(star.screenPosition, viewportSize * 0.5, ghost.position);
    float halfHeight = ghost.size * viewportSize.y * 0.5;
    float2 corner = quadCorners[vertexIndex];
    float2 extent = corner * float2(halfHeight * ghost.aspectRatio, halfHeight);

    // Traced ghosts are baked with their local x axis along the line from the screen center toward the star.
    if (ghost.shape == ghostShapeTexture) {
        float2 fromCenter = star.screenPosition - viewportSize * 0.5;
        float2 axis = length(fromCenter) > 0.0 ? normalize(fromCenter) : float2(1.0, 0.0);
        extent = axis * extent.x + float2(-axis.y, axis.x) * extent.y;
    }

    float2 pixel = center + extent;
    GhostOutput output;
    output.position = float4(pixel / viewportSize * 2.0 - 1.0, 0.0, 1.0);
    output.local = corner;
    output.star = starIndex;
    output.ghost = ghostIndex;
    return output;
}

// 1 on the edge of a regular polygon whose corners touch the unit circle, less inside it.
float polygonDistance(float2 local, uint sides, float rotation) {
    float sector = radians(360.0) / sides;
    float angle = atan2(local.y, local.x) - rotation;
    float fromSectorMiddle = angle - sector * floor(angle / sector) - sector * 0.5;
    return length(local) * cos(fromSectorMiddle) / cos(radians(180.0) / sides);
}

// The ring's look, from its prototype: per-channel color and radius offsets (in units of dispersion), the mix of
// glow and streaks, and where the band fades in and out.
static const float3 ringColor = float3(0.75, 0.55, 0.62);
static const float3 ringDispersionOffsets = float3(-7.0 / 6.0, 0.5, 1.0);
static const float ringGlowWeight = 0.22;
static const float ringStreaksWeight = 0.36;
static const float ringInnerFadeStart = 0.28;
static const float ringInnerFadeWidth = 0.3;
static const float ringOuterFadeEnd = 0.93;
static const float ringOuterFadeWidth = 0.18;
static const float streakReach = 0.11;
static const float streakReachVariation = 0.04;
// Cell counts of the coarser streak noises, relative to the streak count.
static const float coarseStreaksPerStreak = 0.22;
static const float streakReachCellsPerStreak = 0.39;
// Interpolated uniform noise has a standard deviation near 0.5; this brings it to about 1.
static const float noiseNormalization = 2.0;
static const float3 spotGains = float3(1.06, 1.0, 1.11);
static const float3 spotDispersionOffsets = float3(-5.0 / 6.0, 0.0, 5.0 / 6.0);
static const float spotDistanceBeyondRing = 0.17;
static const float2 spotRadius = float2(0.06, 0.05);

float polygonShape(float2 local, GhostData ghost) {
    float distance = polygonDistance(local, constants.blades, constants.rotation);
    return 1.0 - smoothstep(1.0 - ghost.softness, 1.0, distance);
}

uint hash(uint value) {
    value ^= value >> 16;
    value *= 0x7feb352d;
    value ^= value >> 15;
    value *= 0x846ca68b;
    value ^= value >> 16;
    return value;
}

float signedHash(uint cell, uint seed) {
    return float(hash(cell ^ hash(seed))) / 4294967295.0 * 2.0 - 1.0;
}

// Smooth noise around a circle with a whole number of cells, so it joins up at the seam.
float angularNoise(float angle, uint cells, uint seed) {
    float position = angle / radians(360.0) * cells;
    float cell = floor(position);
    float fraction = smoothstep(0.0, 1.0, position - cell);
    uint first = uint(cell + cells) % cells;
    uint second = (first + 1) % cells;
    return lerp(signedHash(first, seed), signedHash(second, seed), fraction) * noiseNormalization;
}

float3 ringShape(float2 local, GhostData ghost) {
    float radius = length(local);
    float angle = atan2(local.y, local.x) + radians(180.0);
    uint coarseCells = max(uint(ghost.streaks * coarseStreaksPerStreak), 1);
    uint reachCells = max(uint(ghost.streaks * streakReachCellsPerStreak), 1);
    float streaks = 0.5 + 0.5 * tanh(0.8 * angularNoise(angle, max(ghost.streaks, 1), 1) +
                                     0.4 * angularNoise(angle, coarseCells, 3));
    float reach = streakReach + streakReachVariation * tanh(angularNoise(angle, reachCells, 2));
    float3 fromRing = radius - (ghost.radius + ghost.dispersion * ringDispersionOffsets);
    float3 glow = exp(-pow(fromRing / ghost.width, 2.0));
    float3 rays = exp(-pow(abs(fromRing) / reach, 2.0)) * (ghost.streaks > 0 ? streaks : 0.0);
    float innerFade = pow(saturate((radius - ringInnerFadeStart) / ringInnerFadeWidth), 2.0);
    float outerFade = saturate((ringOuterFadeEnd - radius) / ringOuterFadeWidth);
    float3 shape = ringColor * (ringGlowWeight * glow + ringStreaksWeight * rays) * innerFade * outerFade;

    for (int side = -1; side <= 1; side += 2) {
        float3 spotCenter = side * (ghost.radius + spotDistanceBeyondRing + ghost.dispersion * spotDispersionOffsets);
        float3 dx = (local.x - spotCenter) / spotRadius.x;
        float dy = local.y / spotRadius.y;
        shape += ghost.spots * spotGains * exp(-dx * dx - dy * dy);
    }

    return shape;
}

[shader("fragment")]
float4 fragmentMain(GhostOutput input) : SV_Target {
    GhostData ghost = constants.ghosts[input.ghost];

    float3 shape;

    if (ghost.shape == ghostShapeTexture) {
        float2 uv = input.local * 0.5 + 0.5;
        shape = textures[NonUniformResourceIndex(ghost.texture)].Sample(clampingSampler, uv).rgb;
    } else if (ghost.shape == ghostShapeRing) {
        shape = ringShape(input.local, ghost);
    } else {
        shape = polygonShape(input.local, ghost);
    }

    ProjectedStarData star = constants.stars[input.star];
    float3 luminance = ghost.intensity * ghost.tint * star.illuminance * star.visibility * shape;
    return float4(luminance * constants.frame->exposure, 1.0);
}
