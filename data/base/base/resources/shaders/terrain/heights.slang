import terrain.cube;
import terrain.tile;

static const int splineTexels = 4;

// A texel of a resident level of the terrain's heights, 0 to 1 over their range, read across face edges.
float heightTexel(TerrainData *terrain, uint level, CubeTexel texel) {
    int extent = int(max(terrain.heightExtent >> level, 1u));
    TexelSource source = texelSource(texel, extent);
    Texture2DArray heights = textureArrays[NonUniformResourceIndex(terrain.heightTexture)];
    int mip = int(level - terrain.heightFirstLevel);
    float sum = 0.0;

    for (uint index = 0; index < source.count; index++) {
        CubeTexel stored = source.texels[index];
        sum += heights.Load(int4(stored.column, stored.row, int(stored.face), mip)).r;
    }

    return sum / float(source.count);
}

// The angle between neighbouring texels of `level`, across a face's middle.
float texelSpacing(TerrainData *terrain, uint level) {
    return pi / 2.0 / float(terrain.heightExtent) * exp2(float(level));
}

// A point of the tile on the texels of `level`: a whole texel index and a position from it, texel centers at whole
// positions. The index of the tile's left and top edge comes from its column, row and depth in integers, exact at
// every depth; only the offset within the tile is a float, so positions keep their precision on the deepest tiles,
// where a float spanning the face would round several vertices onto one position.
struct LevelTexel {
    int2 edge;
    float2 position;
};

LevelTexel levelTexel(TerrainData *terrain, TileData tile, uint level, float2 offset) {
    uint extent = max(terrain.heightExtent >> level, 1u);
    int extentBits = int(firstbithigh(extent));
    uint2 column = uint2(tile.x, tile.y);
    LevelTexel texel;
    float2 edgeFraction = float2(0.0);

    if (extentBits >= int(tile.depth)) {
        texel.edge = int2(column << uint(extentBits - int(tile.depth)));
    } else {
        uint shift = tile.depth - uint(extentBits);
        texel.edge = int2(column >> shift);
        edgeFraction = float2(column & ((1u << shift) - 1u)) * exp2(-float(shift));
    }

    texel.position = edgeFraction + (offset + tile.halfSize) * 0.5 * float(extent) - 0.5;
    return texel;
}

// The cubic B-spline's value and Laplacian (in values per texel squared) over the 4×4 texels around a point, weighted
// here rather than by the sampler, whose bilinear weights have too few bits: they would turn the curve into terraces a
// 256th of a texel wide. Mirrored by `HeightMapLevel::height_at` on the CPU.
struct SplineShape {
    float value;
    float laplacian;
};

SplineShape heightSpline(TerrainData *terrain, TileData tile, uint level, float2 offset) {
    LevelTexel texel = levelTexel(terrain, tile, level, offset);
    float2 base = floor(texel.position);
    float2 f = texel.position - base;
    float2 f2 = f * f;
    float2 f3 = f2 * f;
    float2 weights[4] = {
        (1.0 - 3.0 * f + 3.0 * f2 - f3) / 6.0,
        (4.0 - 6.0 * f2 + 3.0 * f3) / 6.0,
        (1.0 + 3.0 * f + 3.0 * f2 - 3.0 * f3) / 6.0,
        f3 / 6.0,
    };
    float2 curvatures[4] = { 1.0 - f, 3.0 * f - 2.0, 1.0 - 3.0 * f, f };
    int2 first = texel.edge + int2(base) - 1;
    SplineShape shape;
    shape.value = 0.0;
    shape.laplacian = 0.0;

    for (int row = 0; row < splineTexels; row++) {
        float rowValue = 0.0;
        float rowCurvature = 0.0;

        for (int column = 0; column < splineTexels; column++) {
            CubeTexel texel;
            texel.face = tile.face;
            texel.column = first.x + column;
            texel.row = first.y + row;
            float value = heightTexel(terrain, level, texel);
            rowValue += weights[column].x * value;
            rowCurvature += curvatures[column].x * value;
        }

        shape.value += weights[row].y * rowValue;
        shape.laplacian += weights[row].y * rowCurvature + curvatures[row].y * rowValue;
    }

    return shape;
}

// The ground at a point of a level: its height in radii, relative to the radius, and its curvature, the change in
// slope across one texel.
struct LevelShape {
    float height;
    float curvature;
};

LevelShape levelShape(TerrainData *terrain, TileData tile, uint level, float2 offset) {
    SplineShape spline = heightSpline(terrain, tile, level, offset);
    LevelShape shape;
    shape.height = terrain.heightMinimum + spline.value * terrain.heightSpan;
    shape.curvature = spline.laplacian * terrain.heightSpan / texelSpacing(terrain, level);
    return shape;
}

// A tile's height in radii, relative to the radius, at a grid point of a level.
float levelHeight(TerrainData *terrain, TileData tile, uint level, float2 offset) {
    return terrain.heightMinimum + heightSpline(terrain, tile, level, offset).value * terrain.heightSpan;
}

// In radii, relative to the radius: the same function of a vertex's place for every tile of a level, whichever face it
// belongs to, so neighbouring tiles meet.
struct TerrainHeight : TileHeight {
    TerrainData *terrain;

    float height(TileData tile, int2 cell, float morph) {
        TileHeightLevels levels = tileHeightLevels(terrain, tile);
        float2 offset = faceOffset(tile, cell);
        float finer = levelHeight(terrain, tile, levels.finer, offset);

        if (morph == 0.0 || levels.coarser == levels.finer) {
            return finer;
        }

        return lerp(finer, levelHeight(terrain, tile, levels.coarser, offset), morph);
    }

    // The height, the curvature and the height above the ground two levels coarser, over that level's texel spacing:
    // ground below its surroundings reads as low.
    TerrainShape shape(TileData tile, int2 cell, float morph) {
        TileHeightLevels levels = tileHeightLevels(terrain, tile);
        float2 offset = faceOffset(tile, cell);
        LevelShape fine = levelShape(terrain, tile, levels.finer, offset);
        LevelShape coarse = levelShape(terrain, tile, levels.coarser, offset);
        uint surroundingLevel = min(levels.coarser + 2, terrain.heightLevels - 1);
        LevelShape surroundings = levelShape(terrain, tile, surroundingLevel, offset);
        TerrainShape shape;
        shape.height = lerp(fine.height, coarse.height, morph);
        shape.curvature = lerp(fine.curvature, coarse.curvature, morph);
        shape.relativeHeight = (shape.height - surroundings.height) / texelSpacing(terrain, surroundingLevel);
        return shape;
    }
}
