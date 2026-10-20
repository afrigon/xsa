import terrain.tile;

static const int colorLayer = 0;
static const int normalLayer = 1;
static const int mediumLayer = 2;
static const int oppositionLayer = 3;
static const uint finestLevelBits = 5;

// Where a pixel of a tile reads its layers: its face offset from the tile's center, how fast that changes across the
// screen, and the level whose texels are a pixel wide there, within those the tile reads and before that limit.
struct LayerCoordinates {
    TileData tile;
    TilePages *pages;
    float2 offset;
    float2 offsetPerPixelX;
    float2 offsetPerPixelY;
    float level;
    float pixelLevel;
};

// A layer's value at a pixel, and the page it came from: its level and the pixel's place on its texels. Not resident
// when no page covering the pixel is.
struct LayerRead {
    bool resident;
    float4 value;
    uint level;
    float2 texel;
};

LayerCoordinates layerCoordinates(uint tile, float2 offset) {
    LayerCoordinates coordinates;
    coordinates.tile = tileData(tile);
    coordinates.pages = tilePages(tile);
    coordinates.offset = offset;
    coordinates.offsetPerPixelX = ddx(offset);
    coordinates.offsetPerPixelY = ddy(offset);
    float levelZeroTexelsPerPixel =
        max(length(coordinates.offsetPerPixelX), length(coordinates.offsetPerPixelY)) * pageInterior / 2.0;
    float depth = float(coordinates.tile.depth);
    coordinates.pixelLevel = -log2(levelZeroTexelsPerPixel);
    coordinates.level = clamp(coordinates.pixelLevel, depth, depth + float(layerLevels - 1));
    return coordinates;
}

// The finest level a layer has.
uint finestLevel(TileData tile, int layer) {
    return (tile.layerFinestLevels >> (uint(layer) * finestLevelBits)) & ((1u << finestLevelBits) - 1u);
}

// The layer on the page holding the pixel's part of the tile split `finer` levels below it, or a coarser page
// standing in.
LayerRead readLayerAt(LayerCoordinates coordinates, int layer, int finer) {
    uint side = 1u << finer;
    float2 along = (coordinates.offset / (2.0 * coordinates.tile.halfSize) + 0.5) * float(side);
    uint2 part = uint2(clamp(floor(along), 0.0, float(side - 1)));
    int slot = ((1 << (2 * finer)) - 1) / 3 + int(part.y * side + part.x);
    TileLayerPage page = coordinates.pages.layers[layer][slot];
    LayerRead read;
    read.resident = isResident(page);
    read.value = float4(0.0);
    read.level = 0;
    read.texel = float2(0.0);

    if (!read.resident) {
        return read;
    }

    read.level = pageLevel(page);
    read.texel =
        pageTexel(coordinates.tile, read.level, pageKey(coordinates.tile, read.level, finer, part), coordinates.offset);
    Texture2D atlas = textures[NonUniformResourceIndex(pageSlot(page))];
    float2 size;
    atlas.GetDimensions(size.x, size.y);
    float texelsPerOffset = pageInterior / 2.0 * exp2(float(read.level));
    float2 uv = (float2(pageOffset(page)) + read.texel + 0.5) / size;
    read.value = atlas.SampleGrad(wrappingSampler,
                                  uv,
                                  coordinates.offsetPerPixelX * texelsPerOffset / size,
                                  coordinates.offsetPerPixelY * texelsPerOffset / size);
    return read;
}

// A layer blended between the two whole levels around the pixel's, from the finer one's page.
LayerRead readLayer(LayerCoordinates coordinates, int layer) {
    float relative = coordinates.level - float(coordinates.tile.depth);
    int coarser = int(relative);
    int finer = min(coarser + 1, layerLevels - 1);
    LayerRead coarse = readLayerAt(coordinates, layer, coarser);
    LayerRead fine = readLayerAt(coordinates, layer, finer);

    if (!fine.resident) {
        return coarse;
    }

    if (coarse.resident) {
        fine.value = lerp(coarse.value, fine.value, relative - float(coarser));
    }

    return fine;
}

// Normals are tangent space in their face's frame: x along the face's columns and y across them, both on the sphere's
// tangent plane; z is reconstructed. Without a page, the sphere's normal.
float3 layerNormal(LayerRead read, TileData tile, float3 modelNormal) {
    if (!read.resident) {
        return modelNormal;
    }

    float2 slope = read.value.rg * 2.0 - 1.0;
    float3 tangent = normalize(tile.columnAxis - dot(tile.columnAxis, modelNormal) * modelNormal);
    float3 bitangent = cross(tangent, modelNormal);
    float outward = sqrt(saturate(1.0 - dot(slope, slope)));
    return normalize(slope.x * tangent + slope.y * bitangent + outward * modelNormal);
}
