import geometry.tile;

// Heights are in radii, relative to the radius.
struct TerrainData {
    uint heightTexture;
    float minimumHeight;
    float heightRange;
    uint padding;
};

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
struct TerrainSurface : TileSurface {
    float height(TileData tile, ObjectData object, int2 cell) {
        TerrainData terrain = *(TerrainData *)constants.terrain;
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
}

struct TerrainGeometry : Geometry {
    static TileVertex transform(VertexInput input) {
        TerrainSurface surface;
        return transformTile(surface, input);
    }
}
