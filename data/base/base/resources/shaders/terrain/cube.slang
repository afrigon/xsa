import draw;

// The terrain's cube maps, faces in `CubeFace` order: each face's outward normal and the axes its columns and rows run
// along. Mirrors `CubeFace` in resource-loader.
static const int3 faceNormals[6] = {
    int3(1, 0, 0),
    int3(-1, 0, 0),
    int3(0, 1, 0),
    int3(0, -1, 0),
    int3(0, 0, 1),
    int3(0, 0, -1),
};
static const int3 faceColumns[6] = {
    int3(0, 0, -1),
    int3(0, 0, 1),
    int3(1, 0, 0),
    int3(1, 0, 0),
    int3(1, 0, 0),
    int3(-1, 0, 0),
};
static const int3 faceRows[6] = {
    int3(0, -1, 0),
    int3(0, -1, 0),
    int3(0, 0, 1),
    int3(0, 0, -1),
    int3(0, -1, 0),
    int3(0, -1, 0),
};

// Written out rather than `dot`, which on integers needs the integer dot product feature.
int integerDot(int3 first, int3 second) {
    return first.x * second.x + first.y * second.y + first.z * second.z;
}

// The face whose outward normal is a unit axis.
uint faceOfNormal(int3 normal) {
    if (normal.x != 0) {
        return normal.x > 0 ? 0u : 1u;
    }

    if (normal.y != 0) {
        return normal.y > 0 ? 2u : 3u;
    }

    return normal.z > 0 ? 4u : 5u;
}

// The face `direction`, in the body's frame, falls on; ties go to x, then y, as `FacePoint::from_direction` breaks
// them.
uint faceOfDirection(float3 direction) {
    float3 magnitude = abs(direction);

    if (magnitude.x >= magnitude.y && magnitude.x >= magnitude.z) {
        return direction.x >= 0.0 ? 0u : 1u;
    }

    if (magnitude.y >= magnitude.z) {
        return direction.y >= 0.0 ? 2u : 3u;
    }

    return direction.z >= 0.0 ? 4u : 5u;
}

// The direction to sample an equal-angle cube map at toward `direction`: the point of the cube whose plain face
// coordinates are the direction's equal-angle ones. Face edges stay at ±1 in both, so the hardware's seamless
// filtering and level selection work on the stored texels.
float3 equalAngleSampleDirection(float3 direction) {
    uint face = faceOfDirection(direction);
    float3 normal = float3(faceNormals[face]);
    float3 column = float3(faceColumns[face]);
    float3 row = float3(faceRows[face]);
    float2 plane = float2(dot(direction, column), dot(direction, row)) / dot(direction, normal);
    float2 coordinates = atan(plane) * (4.0 / pi);
    return normal + coordinates.x * column + coordinates.y * row;
}

// A texel of a face `extent` texels wide, by column and row; either may lie a few texels beyond the face. Texels are
// placed by exact integer positions: a face spans −extent to extent along its axes, texel centres at the odd
// coordinates. Mirrors `CubeTexel` in core.
struct CubeTexel {
    uint face;
    int column;
    int row;
};

// The stored texels a texel reads: one, or the three a cube corner averages, where no diagonal neighbour exists.
struct TexelSource {
    uint count;
    CubeTexel texels[3];
};

int3 texelPoint(CubeTexel texel, int extent) {
    return faceNormals[texel.face] * extent + faceColumns[texel.face] * (2 * texel.column + 1 - extent) +
           faceRows[texel.face] * (2 * texel.row + 1 - extent);
}

// Halving rounds toward negative infinity, as an arithmetic shift does.
CubeTexel texelAt(uint face, int3 point, int extent) {
    CubeTexel texel;
    texel.face = face;
    texel.column = (integerDot(point, faceColumns[face]) + extent - 1) >> 1;
    texel.row = (integerDot(point, faceRows[face]) + extent - 1) >> 1;
    return texel;
}

// A point beyond a face's edge, folded over that edge onto the adjacent face, whose normal is the edge's direction.
CubeTexel foldedTexel(uint face, int3 point, int3 edge, int extent) {
    int excess = integerDot(point, edge) - extent;
    return texelAt(faceOfNormal(edge), point - edge * excess - faceNormals[face] * excess, extent);
}

TexelSource texelSource(CubeTexel texel, int extent) {
    TexelSource source;
    bool columnInside = texel.column >= 0 && texel.column < extent;
    bool rowInside = texel.row >= 0 && texel.row < extent;

    if (columnInside && rowInside) {
        source.count = 1;
        source.texels[0] = texel;
        return source;
    }

    uint face = texel.face;
    int3 point = texelPoint(texel, extent);
    int3 columnEdge = faceColumns[face] * sign(integerDot(point, faceColumns[face]));
    int3 rowEdge = faceRows[face] * sign(integerDot(point, faceRows[face]));

    if (rowInside) {
        source.count = 1;
        source.texels[0] = foldedTexel(face, point, columnEdge, extent);
        return source;
    }

    if (columnInside) {
        source.count = 1;
        source.texels[0] = foldedTexel(face, point, rowEdge, extent);
        return source;
    }

    int columnExcess = integerDot(point, columnEdge) - extent;
    int rowExcess = integerDot(point, rowEdge) - extent;
    int3 reflected = point - columnEdge * 2 * columnExcess - rowEdge * 2 * rowExcess;
    CubeTexel overColumn = foldedTexel(face, point, columnEdge, extent);
    CubeTexel overRow = foldedTexel(face, point, rowEdge, extent);
    source.count = 3;
    source.texels[0] = texelAt(face, reflected, extent);
    source.texels[1] = foldedTexel(overColumn.face, texelPoint(overColumn, extent), rowEdge, extent);
    source.texels[2] = foldedTexel(overRow.face, texelPoint(overRow, extent), columnEdge, extent);
    return source;
}
