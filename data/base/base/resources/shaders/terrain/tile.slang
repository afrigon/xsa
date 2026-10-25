__exported import draw;

struct VertexInput {
    uint2 cell;
    uint instance : SV_InstanceID;
};

// Everything a tile's vertex stage computes; each terrain program passes on the part its fragment stage reads. The
// face offset is in the face's equal-angle coordinates from the tile's center, which the tile's pages are read at.
struct TileVertex {
    DrawVaryings draw;
    float2 cell;
    int detail;
    uint gridCells;
    float heightMeters;
    uint tile;
    float2 faceOffset;
    // The change in slope across one texel of the heights, and the height above the ground two levels coarser over
    // that level's texel spacing.
    float curvature;
    float relativeHeight;
};

// A quad sphere tile: centers are relative to their frame's camera, the cube center is on the cube spanning -1 to 1,
// the axes are its face's unit column and row axes, and the half size is in the face's equal-angle coordinates.
// `x`, `y` and `depth` are its place in its face's quadtree, which its pages are found from; the finest level of each
// shading layer takes five bits, the first layer lowest.
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
    uint layerFinestLevels;
};

// Where a tile reads one page of a layer: the bindless slot of its atlas chunk with the level of the page it holds,
// which is the page asked for or a coarser one standing in, and the page's texel offset in the chunk.
struct TileLayerPage {
    uint slotAndLevel;
    uint offset;
};

// Heights at one level, with the range the page's 16-bit values span, in radii relative to the radius.
struct TileHeightPage {
    TileLayerPage page;
    float minimum;
    float span;
};

// Mirrors `TilePages` in the renderer: heights at four levels ending one coarser than the tile's depth, and each
// shading layer at the tile's depth and the two levels below it, one page, four, then sixteen, row by row.
static const int heightLevels = 4;
static const int layerLevels = 3;
static const int layerPages = 21;
static const int layers = 4;

struct TilePages {
    TileHeightPage heights[heightLevels];
    TileLayerPage layers[layers][layerPages];
};

static const uint noPage = 0xFFFFFFFF;
static const uint levelShift = 27;
static const float pageInterior = 128.0;
static const float pageBorder = 4.0;

bool isResident(TileLayerPage page) {
    return page.slotAndLevel != noPage;
}

uint pageSlot(TileLayerPage page) {
    return page.slotAndLevel & ((1u << levelShift) - 1u);
}

uint pageLevel(TileLayerPage page) {
    return page.slotAndLevel >> levelShift;
}

int2 pageOffset(TileLayerPage page) {
    return int2(page.offset & 0xFFFFu, page.offset >> 16u);
}

// What every terrain program's fragment stage reads, its debug views included.
struct TerrainVaryings {
    DrawVaryings draw;
    float2 cell;
    nointerpolation int detail;
    nointerpolation uint gridCells;
    float heightMeters;
    nointerpolation uint tile;
    float2 faceOffset;
};

TerrainVaryings terrainVaryings(TileVertex vertex) {
    TerrainVaryings varyings;
    varyings.draw = vertex.draw;
    varyings.cell = vertex.cell;
    varyings.detail = vertex.detail;
    varyings.gridCells = vertex.gridCells;
    varyings.heightMeters = vertex.heightMeters;
    varyings.tile = vertex.tile;
    varyings.faceOffset = vertex.faceOffset;
    return varyings;
}

// A tile's ground at a vertex: its height in radii, relative to the radius, and its shape (see `TileVertex`).
struct TerrainShape {
    float height;
    float curvature;
    float relativeHeight;
};

// The height of a tile's ground above the unit sphere.
interface TileHeight {
    // In radii, relative to the radius.
    float height(TileData tile, ObjectData object, int2 cell);

    TerrainShape shape(TileData tile, ObjectData object, int2 cell);
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

TilePages *tilePages(uint tile) {
    return ((TilePages *)constants.tilePages) + tile;
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

// A point of the tile on the texels of a page at `level` holding it, texel centers at whole coordinates from the
// page's first stored texel. The tile's center comes from its whole column and row, exact at every level.
float2 pageTexel(TileData tile, uint level, uint2 pageKey, float2 offset) {
    float scale = exp2(float(int(level) - int(tile.depth)));
    float2 center =
        (float2(tile.x, tile.y) * 2.0 + 1.0) * (pageInterior / 2.0) * scale - float2(pageKey) * pageInterior;
    return center + offset * (pageInterior / 2.0) * exp2(float(level)) + pageBorder - 0.5;
}

// The key, at its own level, of a page holding the tile's part `part` split `finer` levels below it.
uint2 pageKey(TileData tile, uint level, uint finer, uint2 part) {
    uint2 key = (uint2(tile.x, tile.y) << finer) + part;
    return key >> (tile.depth + finer - level);
}

// From the tile's center to the terrain above a grid point.
float3 terrainOffset<H : TileHeight>(H heights, TileData tile, ObjectData object, int2 cell) {
    float3 direction = normalize(cubePoint(tile, cell));
    return sphereOffset(tile, cell) +
           direction * heights.height(tile, object, cell) * constants.frame.terrainHeightScale;
}

// Towards the end of a tile's range, odd grid points slide to the middle of the edge or diagonal joining their two
// even neighbors: the tile becomes exactly its parent's triangles, and matches neighbors on faces whose grids run
// the other way.
float3 morphedOffset<H : TileHeight>(H heights, TileData tile, ObjectData object, int2 cell) {
    float3 offset = terrainOffset(heights, tile, object, cell);
    float distance = length(tile.center + mul(object.worldFromModel, float4(offset, 0.0)).xyz);
    float morph =
        distance < tile.morphStart ? 0.0 : saturate((distance - tile.morphStart) / (tile.morphEnd - tile.morphStart));
    int2 odd = cell & 1;

    if (morph == 0.0 || all(odd == 0)) {
        return offset;
    }

    float3 parentMidpoint =
        (terrainOffset(heights, tile, object, cell - odd) + terrainOffset(heights, tile, object, cell + odd)) * 0.5;
    return lerp(offset, parentMidpoint, morph);
}

TileVertex transformTile<H : TileHeight, R : TileRelief>(H heights, R relief, VertexInput input) {
    ObjectData object = currentObject();
    uint tileIndex = constants.firstTile + input.instance;
    TileData tile = tileData(tileIndex);
    int2 cell = int2(input.cell);
    float3 offset = morphedOffset(heights, tile, object, cell);
    float3 modelNormal = normalize(normalize(tile.cubeCenter) + offset);
    TerrainShape shape = heights.shape(tile, object, cell);
    float2 vertexFaceOffset = faceOffset(tile, cell);
    float radius = length(object.worldFromModel[0].xyz);
    float3 ground = tile.center + mul(object.worldFromModel, float4(offset, 0.0)).xyz;
    offset += modelNormal * relief.relief(tileIndex, ground, modelNormal, vertexFaceOffset, shape) / radius;
    float3 world = tile.center + mul(object.worldFromModel, float4(offset, 0.0)).xyz;
    float3 previousWorld = tile.previousCenter + mul(object.previousWorldFromModel, float4(offset, 0.0)).xyz;
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
    output.tile = tileIndex;
    output.faceOffset = vertexFaceOffset;
    return output;
}
