__exported import geometry.geometry;

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

// The height of a tile's ground above the unit sphere.
interface TileHeight {
    // In radii, relative to the radius.
    float height(TileData tile, ObjectData object, int2 cell);
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

// From the tile's center to a grid point on the cube. A face coordinate s lies at tan(π/4 s) on the cube; the tangent
// of the center's coordinate plus the offset is built from both tangents, so the small offset keeps its precision.
float3 cubeOffset(TileData tile, int2 cell) {
    float2 centerTangent = float2(dot(tile.cubeCenter, tile.columnAxis), dot(tile.cubeCenter, tile.rowAxis));
    float2 offsetTangent = tan(pi / 4.0 * faceOffset(tile, cell));
    float2 along = offsetTangent * (1.0 + centerTangent * centerTangent) / (1.0 - centerTangent * offsetTangent);
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
    float2 center = (float2(tile.x, tile.y) * 2.0 + 1.0) * (pageInterior / 2.0) * scale - float2(pageKey) * pageInterior;
    return center + offset * (pageInterior / 2.0) * exp2(float(level)) + pageBorder - 0.5;
}

// The key, at its own level, of a page holding the tile's part `part` split `finer` levels below it.
uint2 pageKey(TileData tile, uint level, uint finer, uint2 part) {
    uint2 key = (uint2(tile.x, tile.y) << finer) + part;
    return key >> (tile.depth + finer - level);
}

// From the tile's center to the surface above a grid point.
float3 surfaceOffset<H : TileHeight>(H heights, TileData tile, ObjectData object, int2 cell) {
    float3 direction = normalize(cubePoint(tile, cell));
    return sphereOffset(tile, cell) + direction * heights.height(tile, object, cell);
}

// Towards the end of a tile's range, odd grid points slide to the middle of the edge or diagonal joining their two
// even neighbors: the tile becomes exactly its parent's triangles, and matches neighbors on faces whose grids run
// the other way.
float3 morphedOffset<H : TileHeight>(H heights, TileData tile, ObjectData object, int2 cell) {
    float3 offset = surfaceOffset(heights, tile, object, cell);
    float distance = length(tile.center + mul(object.worldFromModel, float4(offset, 0.0)).xyz);
    float morph =
        distance < tile.morphStart ? 0.0 : saturate((distance - tile.morphStart) / (tile.morphEnd - tile.morphStart));
    int2 odd = cell & 1;

    if (morph == 0.0 || all(odd == 0)) {
        return offset;
    }

    float3 parentMidpoint =
        (surfaceOffset(heights, tile, object, cell - odd) + surfaceOffset(heights, tile, object, cell + odd)) * 0.5;
    return lerp(offset, parentMidpoint, morph);
}

TileVertex transformTile<H : TileHeight>(H heights, VertexInput input) {
    ObjectData object = currentObject();
    uint tileIndex = constants.firstTile + input.instance;
    TileData tile = tileData(tileIndex);
    int2 cell = int2(input.cell);
    float3 offset = morphedOffset(heights, tile, object, cell);
    float3 world = tile.center + mul(object.worldFromModel, float4(offset, 0.0)).xyz;
    float3 previousWorld = tile.previousCenter + mul(object.previousWorldFromModel, float4(offset, 0.0)).xyz;
    float3 modelNormal = normalize(normalize(tile.cubeCenter) + offset);
    TileVertex output;
    output.surface.position = mul(constants.frame.viewProjection, float4(world, 1.0));
    output.surface.currentClip = unjittered(output.surface.position);
    output.surface.previousClip = mul(constants.frame.previousViewProjection, float4(previousWorld, 1.0));
    output.surface.worldPosition = world;
    output.surface.normal = mul(object.worldFromModel, float4(modelNormal, 0.0)).xyz;
    output.surface.modelNormal = modelNormal;
    output.cell = float2(input.cell);
    output.detail = tile.detail;
    output.gridCells = tile.gridCells;
    output.heightMeters = heights.height(tile, object, cell) * length(object.worldFromModel[0].xyz);
    output.tile = tileIndex;
    output.faceOffset = faceOffset(tile, cell);
    return output;
}
