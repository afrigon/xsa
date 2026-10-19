__exported import common.scene;

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
    // The center's position on the height map's top level, split so the fraction keeps full precision.
    int2 centerTexel;
    float2 centerTexelFraction;
};

struct VertexInput {
    uint2 cell;
    uint instance : SV_InstanceID;
};

// Fragment stages read the surface alone or all of it: the surface comes first, so its locations are the same in both.
struct TileVertexOutput {
    VertexOutput surface;
    float2 cell;
    nointerpolation int detail;
    nointerpolation uint gridCells;
    float heightMeters;
};

// The height of a tile's surface above the unit sphere.
interface ITileSurface {
    // In radii, relative to the radius.
    float height(TileData tile, ObjectData object, int2 cell);
}

TileData currentTile(uint instance) {
    return ((TileData *)constants.tiles)[constants.firstTile + instance];
}

float3 cubeOffset(TileData tile, int2 cell) {
    float2 face = float2(cell) / float(tile.gridCells) * 2.0 - 1.0;
    return face.x * tile.horizontalAxis + face.y * tile.verticalAxis;
}

// Exact in f32 down to the deepest tiles, so tiles sharing a grid point compute it bit for bit alike.
float3 cubePoint(TileData tile, int2 cell) {
    return tile.cubeCenter + cubeOffset(tile, cell);
}

// From the tile's center to the point of the unit sphere above a grid point. Built from small differences only, so
// its rounding error scales with the tile rather than the sphere.
float3 sphereOffset(TileData tile, int2 cell) {
    float3 alongFace = cubeOffset(tile, cell);
    float centerLengthSquared = dot(tile.cubeCenter, tile.cubeCenter);
    float growth = 2.0 * dot(tile.cubeCenter, alongFace) + dot(alongFace, alongFace);
    float centerLength = sqrt(centerLengthSquared);
    float pointLength = sqrt(centerLengthSquared + growth);
    return alongFace / pointLength -
           tile.cubeCenter * (growth / (centerLength * pointLength * (centerLength + pointLength)));
}

// From the tile's center to the surface above a grid point.
float3 surfaceOffset<S : ITileSurface>(S surface, TileData tile, ObjectData object, int2 cell) {
    float3 direction = normalize(cubePoint(tile, cell));
    return sphereOffset(tile, cell) + direction * surface.height(tile, object, cell);
}

// Towards the end of a tile's range, odd grid points slide to the middle of the edge or diagonal joining their two
// even neighbors: the tile becomes exactly its parent's triangles, and matches neighbors on faces whose grids run
// the other way.
float3 morphedOffset<S : ITileSurface>(S surface, TileData tile, ObjectData object, int2 cell) {
    float3 offset = surfaceOffset(surface, tile, object, cell);
    float distance = length(tile.center + mul(object.worldFromModel, float4(offset, 0.0)).xyz);
    float morph =
        distance < tile.morphStart ? 0.0 : saturate((distance - tile.morphStart) / (tile.morphEnd - tile.morphStart));
    int2 odd = cell & 1;

    if (morph == 0.0 || all(odd == 0)) {
        return offset;
    }

    float3 parentMidpoint =
        (surfaceOffset(surface, tile, object, cell - odd) + surfaceOffset(surface, tile, object, cell + odd)) * 0.5;
    return lerp(offset, parentMidpoint, morph);
}

TileVertexOutput transformTile<S : ITileSurface>(S surface, VertexInput input) {
    ObjectData object = currentObject();
    TileData tile = currentTile(input.instance);
    int2 cell = int2(input.cell);
    float3 offset = morphedOffset(surface, tile, object, cell);
    float3 world = tile.center + mul(object.worldFromModel, float4(offset, 0.0)).xyz;
    float3 previousWorld = tile.previousCenter + mul(object.previousWorldFromModel, float4(offset, 0.0)).xyz;
    float3 modelNormal = normalize(normalize(tile.cubeCenter) + offset);
    TileVertexOutput output;
    output.surface.position = mul(constants.frame.viewProjection, float4(world, 1.0));
    output.surface.currentClip = unjittered(output.surface.position);
    output.surface.previousClip = mul(constants.frame.previousViewProjection, float4(previousWorld, 1.0));
    output.surface.worldPosition = world;
    output.surface.normal = mul(object.worldFromModel, float4(modelNormal, 0.0)).xyz;
    output.surface.modelNormal = modelNormal;
    output.cell = float2(input.cell);
    output.detail = tile.detail;
    output.gridCells = tile.gridCells;
    output.heightMeters = surface.height(tile, object, cell) * length(object.worldFromModel[0].xyz);
    return output;
}
