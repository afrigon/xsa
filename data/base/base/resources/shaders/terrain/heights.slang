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

// A point of the tile on the texels of `level`, texel centers at whole coordinates. The tile's center comes from its
// whole column and row, exact at every depth, and is not taken back to the face's center before the offset is added.
float2 levelTexel(TerrainData *terrain, TileData tile, uint level, float2 offset) {
    float extent = float(max(terrain.heightExtent >> level, 1u));
    float2 fromCorner = (float2(tile.x, tile.y) * 2.0 + 1.0) * tile.halfSize + offset;
    return fromCorner * 0.5 * extent - 0.5;
}

// The cubic B-spline's value and Laplacian (in values per texel squared) over the 4×4 texels around a point, weighted
// here rather than by the sampler, whose bilinear weights have too few bits: they would turn the curve into terraces a
// 256th of a texel wide. Mirrored by `HeightMapLevel::height_at` on the CPU.
struct SplineShape {
    float value;
    float laplacian;
};

SplineShape heightSpline(TerrainData *terrain, TileData tile, uint level, float2 offset) {
    float2 texel = levelTexel(terrain, tile, level, offset);
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
    float2 curvatures[4] = { 1.0 - f, 3.0 * f - 2.0, 1.0 - 3.0 * f, f };
    int2 first = int2(base) - 1;
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
