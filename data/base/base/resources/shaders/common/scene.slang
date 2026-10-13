__exported import common.frame;

struct ObjectData {
    float4x4 worldFromModel;
    float4x4 previousWorldFromModel;
};

// A quad sphere tile: centers are relative to their frame's camera, cube coordinates are on the cube spanning -1 to 1.
struct TileData {
    float3 center;
    int detail;
    float3 previousCenter;
    uint gridCells;
    float3 cubeCenter;
    float morphStart;
    float3 horizontalAxis;
    float morphEnd;
    float3 verticalAxis;
    uint padding;
};

struct PushConstants {
    FrameData *frame;
    ObjectData *objects;
    void *material;
    uint objectIndex;
    uint firstTile;
    TileData *tiles;
};

[[vk::push_constant]]
ConstantBuffer<PushConstants> constants;

static const float pi = 3.14159265358979;
static const float uvPerNdc = 0.5;

struct VertexInput {
    uint2 cell;
    uint instance : SV_InstanceID;
};

struct VertexOutput {
    float4 position : SV_Position;
    float3 worldPosition;
    float3 normal;
    float3 modelNormal;
    float4 currentClip;
    float4 previousClip;
};

struct SurfaceOutput {
    float4 color : SV_Target0;
    float2 motion : SV_Target1;
};

// The jitter shifts every clip-space position by the same fraction of the viewport.
float4 unjittered(float4 clip) {
    return float4(clip.xy - constants.frame.jitter * clip.w, clip.zw);
}

// How far the surface moved across the screen since the previous frame, in texture coordinates.
float2 motionVector(float4 currentClip, float4 previousClip) {
    float2 current = currentClip.xy / currentClip.w;
    float2 previous = previousClip.xy / previousClip.w;
    return (current - previous) * uvPerNdc;
}

SurfaceOutput surfaceOutput(float4 color, VertexOutput input) {
    SurfaceOutput output;
    output.color = color;
    output.motion = motionVector(input.currentClip, input.previousClip);
    return output;
}

ObjectData currentObject() {
    return constants.objects[constants.objectIndex];
}

T currentMaterial<T>() {
    return *(T *)constants.material;
}

TileData currentTile(uint instance) {
    return constants.tiles[constants.firstTile + instance];
}

// From the tile's center to the point of the unit sphere above a grid point. Built from small differences only, so
// its rounding error scales with the tile rather than the sphere.
float3 sphereOffset(TileData tile, int2 cell) {
    float2 face = float2(cell) / float(tile.gridCells) * 2.0 - 1.0;
    float3 cubeOffset = face.x * tile.horizontalAxis + face.y * tile.verticalAxis;
    float centerLengthSquared = dot(tile.cubeCenter, tile.cubeCenter);
    float growth = 2.0 * dot(tile.cubeCenter, cubeOffset) + dot(cubeOffset, cubeOffset);
    float centerLength = sqrt(centerLengthSquared);
    float pointLength = sqrt(centerLengthSquared + growth);
    return cubeOffset / pointLength -
           tile.cubeCenter * (growth / (centerLength * pointLength * (centerLength + pointLength)));
}

// Towards the end of a tile's range, odd grid points slide to the middle of the edge or diagonal joining their two
// even neighbors: the tile becomes exactly its parent's triangles, and matches neighbors on faces whose grids run
// the other way.
float3 morphedOffset(TileData tile, ObjectData object, int2 cell) {
    float3 offset = sphereOffset(tile, cell);
    float distance = length(tile.center + mul(object.worldFromModel, float4(offset, 0.0)).xyz);
    float morph =
        distance < tile.morphStart ? 0.0 : saturate((distance - tile.morphStart) / (tile.morphEnd - tile.morphStart));
    int2 odd = cell & 1;

    if (morph == 0.0 || all(odd == 0)) {
        return offset;
    }

    float3 parentMidpoint = (sphereOffset(tile, cell - odd) + sphereOffset(tile, cell + odd)) * 0.5;
    return lerp(offset, parentMidpoint, morph);
}

VertexOutput transformVertex(VertexInput input) {
    ObjectData object = currentObject();
    TileData tile = currentTile(input.instance);
    float3 offset = morphedOffset(tile, object, int2(input.cell));
    float3 world = tile.center + mul(object.worldFromModel, float4(offset, 0.0)).xyz;
    float3 previousWorld = tile.previousCenter + mul(object.previousWorldFromModel, float4(offset, 0.0)).xyz;
    float3 modelNormal = normalize(normalize(tile.cubeCenter) + offset);
    VertexOutput output;
    output.position = mul(constants.frame.viewProjection, float4(world, 1.0));
    output.currentClip = unjittered(output.position);
    output.previousClip = mul(constants.frame.previousViewProjection, float4(previousWorld, 1.0));
    output.worldPosition = world;
    output.normal = mul(object.worldFromModel, float4(modelNormal, 0.0)).xyz;
    output.modelNormal = modelNormal;
    return output;
}

float starCosine(StarData star, float3 worldPosition, float3 normal) {
    float3 toStar = normalize(star.positionAndRadius.xyz - worldPosition);
    return max(dot(normal, toStar), 0.0);
}

// Lux per color channel: the star's luminous intensity falls off with the square of the distance.
float3 starIlluminance(StarData star, float3 worldPosition) {
    float3 toStar = star.positionAndRadius.xyz - worldPosition;
    return star.intensity.rgb / dot(toStar, toStar);
}

// cd/m² reflected from the faint starlight arriving from every direction.
float3 starlightLuminance(float3 albedo) {
    return albedo / pi * constants.frame.starlightIlluminance;
}

// cd/m² reflected by a Lambertian surface of the given albedo.
float3 lambertLuminance(float3 albedo, float3 illuminance, float cosine) {
    return albedo / pi * illuminance * cosine;
}
