import terrain.tile;

// The angle between neighboring texels of level 0 pages, across a face's middle.
static const float levelZeroTexelAngle = pi / 2.0 / pageInterior;

// A cubic B-spline over the 4×4 texels around `texel` on a page at `offset` in its atlas chunk, weighted here rather
// than by the sampler, whose bilinear weights have too few bits: they would turn the curve into terraces a 256th of
// a texel wide. Mirrored by `HeightPages::height_at` on the CPU.
float bSplineSample(Texture2D atlas, int2 offset, float2 texel) {
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
    int2 first = offset + int2(base) - 1;
    float sum = 0.0;

    for (int row = 0; row < 4; row++) {
        float rowSum = 0.0;

        for (int column = 0; column < 4; column++) {
            rowSum += weights[column].x * atlas.Load(int3(first + int2(column, row), 0)).r;
        }

        sum += weights[row].y * rowSum;
    }

    return sum;
}

// The cubic B-spline's value and Laplacian (in values per texel squared) over the same 4×4 texels, from the second
// derivatives of its weights.
struct SplineShape {
    float value;
    float laplacian;
};

SplineShape bSplineShape(Texture2D atlas, int2 offset, float2 texel) {
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
    int2 first = offset + int2(base) - 1;
    SplineShape shape;
    shape.value = 0.0;
    shape.laplacian = 0.0;

    for (int row = 0; row < 4; row++) {
        float rowValue = 0.0;
        float rowCurvature = 0.0;

        for (int column = 0; column < 4; column++) {
            float texelValue = atlas.Load(int3(first + int2(column, row), 0)).r;
            rowValue += weights[column].x * texelValue;
            rowCurvature += curvatures[column].x * texelValue;
        }

        shape.value += weights[row].y * rowValue;
        shape.laplacian += weights[row].y * rowCurvature + curvatures[row].y * rowValue;
    }

    return shape;
}

// The radii between neighboring texels of `level`, across a face's middle.
float texelSpacing(uint level) {
    return levelZeroTexelAngle * exp2(-float(level));
}

// In radii, relative to the radius; the sphere where no page covering the tile is resident.
float pageHeight(TileData tile, TileHeightPage height, float2 offset) {
    if (!isResident(height.page)) {
        return 0.0;
    }

    uint level = pageLevel(height.page);
    uint2 key = uint2(tile.x, tile.y) >> (tile.depth - level);
    Texture2D atlas = textures[NonUniformResourceIndex(pageSlot(height.page))];
    float value = bSplineSample(atlas, pageOffset(height.page), pageTexel(tile, level, key, offset));
    return height.minimum + value * height.span;
}

// The ground at a point of a page: its height in radii, relative to the radius, and its curvature, the change in slope
// across one texel. Not resident when no page covering the tile is.
struct PageShape {
    bool resident;
    float height;
    float curvature;
    uint level;
};

PageShape pageShape(TileData tile, TileHeightPage height, float2 offset) {
    PageShape shape;
    shape.resident = isResident(height.page);
    shape.height = 0.0;
    shape.curvature = 0.0;
    shape.level = 0;

    if (!shape.resident) {
        return shape;
    }

    shape.level = pageLevel(height.page);
    uint2 key = uint2(tile.x, tile.y) >> (tile.depth - shape.level);
    Texture2D atlas = textures[NonUniformResourceIndex(pageSlot(height.page))];
    SplineShape spline = bSplineShape(atlas, pageOffset(height.page), pageTexel(tile, shape.level, key, offset));
    shape.height = height.minimum + spline.value * height.span;
    shape.curvature = spline.laplacian * height.span / texelSpacing(shape.level);
    return shape;
}

// In radii, relative to the radius. The level follows the vertex spacing at this distance, so neighboring tiles and
// detail levels sample the same level where they meet; it blends the two whole levels around it.
struct TerrainHeight : TileHeight {
    TilePages *pages;

    float height(TileData tile, ObjectData object, int2 cell) {
        float radius = length(object.worldFromModel[0].xyz);
        float distance = length(tile.center + mul(object.worldFromModel, float4(sphereOffset(tile, cell), 0.0)).xyz);
        float vertexSpacing = distance / radius * constants.frame.vertexSpacingAngle;
        float level = log2(levelZeroTexelAngle / vertexSpacing);
        float slot = clamp(level - float(int(tile.depth) - heightLevels), 0.0, float(heightLevels - 1));
        int coarser = int(slot);
        int finer = min(coarser + 1, heightLevels - 1);
        float2 offset = faceOffset(tile, cell);
        return lerp(pageHeight(tile, pages.heights[coarser], offset),
                    pageHeight(tile, pages.heights[finer], offset),
                    slot - float(coarser));
    }

    // The height, the curvature and the height above the ground two levels coarser, over that level's texel spacing:
    // ground below its surroundings reads as low.
    TerrainShape shape(TileData tile, ObjectData object, int2 cell) {
        float radius = length(object.worldFromModel[0].xyz);
        float distance = length(tile.center + mul(object.worldFromModel, float4(sphereOffset(tile, cell), 0.0)).xyz);
        float vertexSpacing = distance / radius * constants.frame.vertexSpacingAngle;
        float level = log2(levelZeroTexelAngle / vertexSpacing);
        float slot = clamp(level - float(int(tile.depth) - heightLevels), 0.0, float(heightLevels - 1));
        int coarser = int(slot);
        int finer = min(coarser + 1, heightLevels - 1);
        float2 offset = faceOffset(tile, cell);
        PageShape coarse = pageShape(tile, pages.heights[coarser], offset);
        PageShape fine = pageShape(tile, pages.heights[finer], offset);
        PageShape surroundings = pageShape(tile, pages.heights[max(coarser - 2, 0)], offset);
        float blend = slot - float(coarser);
        TerrainShape shape;
        shape.height = lerp(coarse.height, fine.height, blend);
        shape.curvature = lerp(coarse.curvature, fine.curvature, blend);
        shape.relativeHeight =
            surroundings.resident ? (shape.height - surroundings.height) / texelSpacing(surroundings.level) : 0.0;
        return shape;
    }
}
