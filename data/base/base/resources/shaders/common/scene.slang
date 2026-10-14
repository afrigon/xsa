__exported import common.frame;

[[vk::binding(2, 0)]]
SamplerState wrappingSampler;

[[vk::binding(3, 0)]]
Texture2D textures[];

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
    // The center's position on the height map's top level, split so the fraction keeps full precision.
    int2 centerTexel;
    float2 centerTexelFraction;
};

// Heights are in radii, relative to the radius.
struct TerrainData {
    uint heightTexture;
    float minimumHeight;
    float heightRange;
    uint padding;
};

struct PushConstants {
    FrameData *frame;
    ObjectData *objects;
    void *material;
    uint objectIndex;
    uint firstTile;
    TileData *tiles;
    // Null for a smooth body.
    TerrainData *terrain;
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

// A point on the height map's top level, texel centers at whole coordinates: whole texels apart from the fraction,
// which f32 could not keep beside a coordinate in the thousands.
struct TexelPosition {
    int2 whole;
    float2 fraction;
};

// The tile's center comes exact from the CPU; the grid point adds its change of longitude and latitude, built from
// the small offset to the center only so it keeps its precision. Longitude 0 sits a quarter of the way along each
// row, growing westward; rows run from the south pole.
TexelPosition texelPosition(TileData tile, int2 cell, uint2 mapSize) {
    float2 size = float2(mapSize);
    float3 point = cubePoint(tile, cell);

    // The pole has no longitude; like the CPU's atan2(0, 0), it takes longitude 0, the same in every tile.
    if (all(point.xy == 0.0)) {
        float2 pole = float2(0.25 * size.x, point.z > 0.0 ? size.y : 0.0) - 0.5;
        TexelPosition position;
        position.whole = int2(floor(pole));
        position.fraction = pole - floor(pole);
        return position;
    }

    float3 center = normalize(tile.cubeCenter);
    float3 offset = sphereOffset(tile, cell);
    // Longitude changes are measured from the center's meridian, or from longitude 0 for a tile centered on a pole.
    float2 meridian = all(tile.cubeCenter.xy == 0.0) ? float2(1.0, 0.0) : center.xy;
    float longitudeCross =
        (meridian.x * center.y - meridian.y * center.x) + (meridian.x * offset.y - meridian.y * offset.x);
    float longitudeDot = meridian.x * (center.x + offset.x) + meridian.y * (center.y + offset.y);
    float longitudeChange = atan2(longitudeCross, longitudeDot);
    float centerRing = length(center.xy);
    float ringGrowth = 2.0 * dot(center.xy, offset.xy) + dot(offset.xy, offset.xy);
    float pointRing = sqrt(centerRing * centerRing + ringGrowth);
    float ringChange = ringGrowth / (centerRing + pointRing);
    float latitudeChange =
        atan2(offset.z * centerRing - center.z * ringChange, pointRing * centerRing + (center.z + offset.z) * center.z);
    float2 change = float2(-longitudeChange / (2.0 * pi) * size.x, latitudeChange / pi * size.y);
    float2 texel = tile.centerTexelFraction + change;
    float2 whole = floor(texel);
    TexelPosition position;
    position.whole = tile.centerTexel + int2(whole);
    position.whole.x = ((position.whole.x % int(mapSize.x)) + int(mapSize.x)) % int(mapSize.x);
    position.fraction = texel - whole;
    return position;
}

// A cubic B-spline over the 4×4 texels around `position` on mip `level`, weighted here rather than by the sampler,
// whose bilinear weights have too few bits: they would turn the curve into terraces a 256th of a texel wide. Mirrored
// by HeightMap::height_at on the CPU.
float bSplineSample(Texture2D texture, TexelPosition position, uint level) {
    uint width;
    uint height;
    uint levels;
    texture.GetDimensions(level, width, height, levels);
    int2 coarse = position.whole >> int(level);
    int2 remainder = position.whole - (coarse << int(level));
    float2 texel = (float2(remainder) + position.fraction + 0.5) / float(1u << level) - 0.5;
    float2 base = floor(texel);
    float2 f = texel - base;
    float2 f2 = f * f;
    float2 f3 = f2 * f;
    float2 weights[4] = {
        (1.0 - 3.0 * f + 3.0 * f2 - f3) / 6.0,
        (4.0 - 6.0 * f2 + 3.0 * f3) / 6.0,
        (1.0 + 3.0 * f + 3.0 * f2 - 3.0 * f3) / 6.0,
        f3 / 6.0,
    };
    int2 first = coarse + int2(base) - 1;
    float sum = 0.0;

    for (int row = 0; row < 4; row++) {
        int y = clamp(first.y + row, 0, int(height) - 1);
        float rowSum = 0.0;

        for (int column = 0; column < 4; column++) {
            int x = (first.x + column) % int(width);
            x = x < 0 ? x + int(width) : x;
            rowSum += weights[column].x * texture.Load(int3(x, y, int(level))).r;
        }

        sum += weights[row].y * rowSum;
    }

    return sum;
}

// In radii, relative to the radius. The mip level follows the vertex spacing at this distance, so neighboring tiles
// and detail levels sample the same level where they meet.
float surfaceHeight(TileData tile, ObjectData object, int2 cell) {
    if (constants.terrain == nullptr) {
        return 0.0;
    }

    TerrainData terrain = *constants.terrain;
    Texture2D heightMap = textures[NonUniformResourceIndex(terrain.heightTexture)];
    uint width;
    uint height;
    uint levels;
    heightMap.GetDimensions(0, width, height, levels);
    float radius = length(object.worldFromModel[0].xyz);
    float distance = length(tile.center + mul(object.worldFromModel, float4(sphereOffset(tile, cell), 0.0)).xyz);
    float vertexSpacing = distance / radius * constants.frame.vertexSpacingAngle;
    float texelSpacing = 2.0 * pi / float(width);
    float level = clamp(log2(vertexSpacing / texelSpacing), 0.0, float(levels - 1));
    uint fineLevel = uint(level);
    uint coarseLevel = min(fineLevel + 1, levels - 1);
    TexelPosition position = texelPosition(tile, cell, uint2(width, height));
    float value = lerp(bSplineSample(heightMap, position, fineLevel),
                       bSplineSample(heightMap, position, coarseLevel),
                       level - float(fineLevel));
    return terrain.minimumHeight + value * terrain.heightRange;
}

// From the tile's center to the terrain's surface above a grid point.
float3 surfaceOffset(TileData tile, ObjectData object, int2 cell) {
    float3 direction = normalize(cubePoint(tile, cell));
    return sphereOffset(tile, cell) + direction * surfaceHeight(tile, object, cell);
}

// Towards the end of a tile's range, odd grid points slide to the middle of the edge or diagonal joining their two
// even neighbors: the tile becomes exactly its parent's triangles, and matches neighbors on faces whose grids run
// the other way.
float3 morphedOffset(TileData tile, ObjectData object, int2 cell) {
    float3 offset = surfaceOffset(tile, object, cell);
    float distance = length(tile.center + mul(object.worldFromModel, float4(offset, 0.0)).xyz);
    float morph =
        distance < tile.morphStart ? 0.0 : saturate((distance - tile.morphStart) / (tile.morphEnd - tile.morphStart));
    int2 odd = cell & 1;

    if (morph == 0.0 || all(odd == 0)) {
        return offset;
    }

    float3 parentMidpoint = (surfaceOffset(tile, object, cell - odd) + surfaceOffset(tile, object, cell + odd)) * 0.5;
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
