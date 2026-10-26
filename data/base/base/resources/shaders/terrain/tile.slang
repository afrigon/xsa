__exported import draw;

struct VertexInput {
    uint2 cell;
    uint instance : SV_InstanceID;
};

// Everything a tile's vertex stage computes; each terrain program passes on the part its fragment stage reads.
struct TileVertex {
    DrawVaryings draw;
    float2 cell;
    int detail;
    uint gridCells;
    float heightMeters;
    // The change in slope across one texel of the heights, and the height above the ground two levels coarser over
    // that level's texel spacing.
    float curvature;
    float relativeHeight;
    // The height level the vertex reads, its blend toward the coarser one included, and the level matching its tile.
    float heightLevel;
    int matchingHeightLevel;
};

// A quad sphere tile: centers are relative to their frame's camera, the cube center is on the cube spanning -1 to 1,
// the axes are its face's unit column and row axes, and the half size is in the face's equal-angle coordinates.
// `face`, `x`, `y` and `depth` are its place in its face's quadtree, which its heights are read at.
struct TileData {
    float3 center;
    int detail;
    float3 previousCenter;
    uint gridCells;
    float3 cubeCenter;
    float morphStart;
    float3 columnAxis;
    float morphEnd;
    float3 rowAxis;
    float halfSize;
    uint x;
    uint y;
    uint depth;
    uint face;
};

// A terrain's maps, mirroring `TerrainData` in the renderer. Slots are `noTexture` for a map the terrain lacks or has
// not loaded; heights are in radii relative to the radius, from their level 0, `heightExtent` texels wide, through
// `heightLevels` levels, of which those from `heightFirstLevel` are resident.
struct TerrainData {
    uint heightTexture;
    uint heightExtent;
    uint heightLevels;
    uint heightFirstLevel;
    float heightMinimum;
    float heightSpan;
    uint layerTextures[4];
    uint colorExtent;
    uint padding;
};

// What every terrain program's fragment stage reads, its debug views included.
struct TerrainVaryings {
    DrawVaryings draw;
    float2 cell;
    nointerpolation int detail;
    nointerpolation uint gridCells;
    float heightMeters;
    float heightLevel;
    nointerpolation int matchingHeightLevel;
};

TerrainVaryings terrainVaryings(TileVertex vertex) {
    TerrainVaryings varyings;
    varyings.draw = vertex.draw;
    varyings.cell = vertex.cell;
    varyings.detail = vertex.detail;
    varyings.gridCells = vertex.gridCells;
    varyings.heightMeters = vertex.heightMeters;
    varyings.heightLevel = vertex.heightLevel;
    varyings.matchingHeightLevel = vertex.matchingHeightLevel;
    return varyings;
}

// A tile's ground at a vertex: its height in radii, relative to the radius, and its shape (see `TileVertex`).
struct TerrainShape {
    float height;
    float curvature;
    float relativeHeight;
};

// The height of a tile's ground above the unit sphere, in radii relative to the radius: at the tile's own level,
// blended toward its parent's by `morph`.
interface TileHeight {
    float height(TileData tile, int2 cell, float morph);

    TerrainShape shape(TileData tile, int2 cell, float morph);
}

// The height levels a tile reads: the one whose texels are as far apart as its vertices, which it shows, and the next
// coarser, its parent's, which it morphs toward; both within the resident levels. `matching` is the level its vertices
// would read with every level resident. Mirrored by `DetailRanges::height_level`.
struct TileHeightLevels {
    uint finer;
    uint coarser;
    int matching;
};

TileHeightLevels tileHeightLevels(TerrainData *terrain, TileData tile) {
    int extentLevels = int(firstbithigh(max(terrain.heightExtent, 1u)));
    int gridLevels = int(firstbithigh(tile.gridCells));
    int coarsest = int(terrain.heightLevels) - 1;
    TileHeightLevels levels;
    levels.matching = extentLevels - int(tile.depth) - gridLevels;
    levels.finer = uint(clamp(levels.matching, int(terrain.heightFirstLevel), coarsest));
    levels.coarser = uint(clamp(levels.matching + 1, int(terrain.heightFirstLevel), coarsest));
    return levels;
}

// How far a tile's ground moves along the sphere's normal beyond its heights (m), at a vertex `world` from the camera
// and `faceOffset` from its tile's center, where the heights have `shape`.
interface TileRelief {
    float relief(uint tile, float3 world, float3 modelNormal, float2 faceOffset, TerrainShape shape);
}

struct NoRelief : TileRelief {
    float relief(uint tile, float3 world, float3 modelNormal, float2 faceOffset, TerrainShape shape) {
        return 0.0;
    }
}

TileData tileData(uint tile) {
    return ((TileData *)constants.tiles)[tile];
}

TerrainData *terrainData() {
    return (TerrainData *)constants.terrain;
}

// Exact in f32 down to the deepest tiles, so tiles sharing a grid point find it alike.
float2 faceOffset(TileData tile, int2 cell) {
    return (float2(cell) / float(tile.gridCells) * 2.0 - 1.0) * tile.halfSize;
}

// GPU tangents are only accurate to an absolute 2^-11, which swamps the tiny angles of deep tiles' offsets; below
// this angle, the Taylor series to a^9 is exact in f32.
static const float seriesTangentLimit = 0.25;

float2 offsetTangent(float2 angle) {
    float2 square = angle * angle;
    float2 series =
        angle * (1.0 + square * (1.0 / 3.0 + square * (2.0 / 15.0 + square * (17.0 / 315.0 + square * 62.0 / 2835.0))));
    return select(abs(angle) < seriesTangentLimit, series, tan(angle));
}

// From the tile's center to a grid point on the cube. A face coordinate s lies at tan(π/4 s) on the cube; the tangent
// of the center's coordinate plus the offset is built from both tangents, so the small offset keeps its precision.
float3 cubeOffset(TileData tile, int2 cell) {
    float2 centerTangent = float2(dot(tile.cubeCenter, tile.columnAxis), dot(tile.cubeCenter, tile.rowAxis));
    float2 tangent = offsetTangent(pi / 4.0 * faceOffset(tile, cell));
    float2 along = tangent * (1.0 + centerTangent * centerTangent) / (1.0 - centerTangent * tangent);
    return along.x * tile.columnAxis + along.y * tile.rowAxis;
}

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

// From the tile's center to the terrain above a grid point, its height blended toward the parent level by `morph`.
float3 terrainOffset<H : TileHeight>(H heights, TileData tile, int2 cell, float morph) {
    float3 direction = normalize(cubePoint(tile, cell));
    return sphereOffset(tile, cell) +
           direction * heights.height(tile, cell, morph) * constants.frame.terrainHeightScale;
}

// How far toward its parent's grid a vertex has morphed, by its distance with the height of the tile's own level.
float vertexMorph<H : TileHeight>(H heights, TileData tile, ObjectData object, int2 cell) {
    float3 offset = terrainOffset(heights, tile, cell, 0.0);
    float distance = length(tile.center + mul(object.worldFromModel, float4(offset, 0.0)).xyz);
    return distance < tile.morphStart ? 0.0
                                      : saturate((distance - tile.morphStart) / (tile.morphEnd - tile.morphStart));
}

// Towards the end of a tile's range, its heights blend into its parent level's and odd grid points slide to the
// middle of the edge or diagonal joining their two even neighbors: the tile becomes exactly its parent's triangles,
// and matches neighbors on faces whose grids run the other way.
float3 morphedOffset<H : TileHeight>(H heights, TileData tile, int2 cell, float morph) {
    float3 offset = terrainOffset(heights, tile, cell, morph);
    int2 odd = cell & 1;

    if (morph == 0.0 || all(odd == 0)) {
        return offset;
    }

    float3 parentMidpoint =
        (terrainOffset(heights, tile, cell - odd, morph) + terrainOffset(heights, tile, cell + odd, morph)) * 0.5;
    return lerp(offset, parentMidpoint, morph);
}

TileVertex transformTile<H : TileHeight, R : TileRelief>(H heights, R relief, VertexInput input) {
    ObjectData object = currentObject();
    uint tileIndex = constants.firstTile + input.instance;
    TileData tile = tileData(tileIndex);
    int2 cell = int2(input.cell);
    float morph = vertexMorph(heights, tile, object, cell);
    float3 offset = morphedOffset(heights, tile, cell, morph);
    float3 modelNormal = normalize(normalize(tile.cubeCenter) + offset);
    TerrainShape shape = heights.shape(tile, cell, morph);
    float2 vertexFaceOffset = faceOffset(tile, cell);
    float radius = length(object.worldFromModel[0].xyz);
    float3 ground = tile.center + mul(object.worldFromModel, float4(offset, 0.0)).xyz;
    offset += modelNormal * relief.relief(tileIndex, ground, modelNormal, vertexFaceOffset, shape) / radius;
    float3 world = tile.center + mul(object.worldFromModel, float4(offset, 0.0)).xyz;
    float3 previousWorld = tile.previousCenter + mul(object.previousWorldFromModel, float4(offset, 0.0)).xyz;
    TileHeightLevels levels = tileHeightLevels(terrainData(), tile);
    TileVertex output;
    output.draw.position = mul(constants.frame.viewProjection, float4(world, 1.0));
    output.draw.currentClip = unjittered(output.draw.position);
    output.draw.previousClip = mul(constants.frame.previousViewProjection, float4(previousWorld, 1.0));
    output.draw.worldPosition = world;
    output.draw.normal = mul(object.worldFromModel, float4(modelNormal, 0.0)).xyz;
    output.draw.modelNormal = modelNormal;
    output.cell = float2(input.cell);
    output.detail = tile.detail;
    output.gridCells = tile.gridCells;
    output.heightMeters = shape.height * radius;
    output.curvature = shape.curvature;
    output.relativeHeight = shape.relativeHeight;
    output.heightLevel = lerp(float(levels.finer), float(levels.coarser), morph);
    output.matchingHeightLevel = levels.matching;
    return output;
}
