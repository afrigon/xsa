import common.frame;
import common.lens;

[[vk::push_constant]]
ConstantBuffer<LensConstants> constants;

static const float2 quadCorners[6] = {
    float2(-1.0, -1.0), float2(1.0, -1.0), float2(1.0, 1.0),
    float2(-1.0, -1.0), float2(1.0, 1.0), float2(-1.0, 1.0),
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
    float2 pixel = center + corner * float2(halfHeight * ghost.aspectRatio, halfHeight);
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

[shader("fragment")]
float4 fragmentMain(GhostOutput input) : SV_Target {
    GhostData ghost = constants.ghosts[input.ghost];

    if (ghost.shape != ghostShapePolygon) {
        discard;
    }

    float distance = polygonDistance(input.local, constants.blades, constants.rotation);
    float shape = 1.0 - smoothstep(1.0 - ghost.softness, 1.0, distance);
    ProjectedStarData star = constants.stars[input.star];
    float3 luminance = ghost.intensity * ghost.tint * star.illuminance * constants.visibility[input.star] * shape;
    return float4(luminance * constants.frame->exposure, 1.0);
}
