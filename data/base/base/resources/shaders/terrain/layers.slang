import terrain.cube;
import terrain.tile;

[[vk::binding(1, 0)]]
TextureCube cubeTextures[];

static const int colorLayer = 0;
static const int normalLayer = 1;
static const int scatteringLayer = 2;
static const int surgeLayer = 3;

// Where a point of the terrain reads its layers: the direction to sample their equal-angle cube maps at, and the face
// it falls on, whose frame the normal layer is stored in.
struct LayerCoordinates {
    TerrainData *terrain;
    float3 sampleDirection;
    uint face;
};

// A layer's value at a point; not resident when the terrain lacks the layer or has not loaded it.
struct LayerRead {
    bool resident;
    float4 value;
};

// `direction` is the point's unit direction in the body's frame.
LayerCoordinates layerCoordinates(float3 direction) {
    LayerCoordinates coordinates;
    coordinates.terrain = terrainData();
    coordinates.sampleDirection = equalAngleSampleDirection(direction);
    coordinates.face = faceOfDirection(direction);
    return coordinates;
}

LayerRead unreadLayer() {
    LayerRead read;
    read.resident = false;
    read.value = float4(0.0);
    return read;
}

// Filtered across the screen as the hardware picks the levels, in a fragment stage.
LayerRead readLayer(LayerCoordinates coordinates, int layer) {
    uint texture = coordinates.terrain.layerTextures[layer];

    if (texture == noTexture) {
        return unreadLayer();
    }

    LayerRead read;
    read.resident = true;
    read.value = cubeTextures[NonUniformResourceIndex(texture)].Sample(wrappingSampler, coordinates.sampleDirection);
    return read;
}

// The finest resident level, for a vertex near the camera, where the ground magnifies the layer.
LayerRead readMagnifiedLayer(LayerCoordinates coordinates, int layer) {
    uint texture = coordinates.terrain.layerTextures[layer];

    if (texture == noTexture) {
        return unreadLayer();
    }

    LayerRead read;
    read.resident = true;
    read.value =
        cubeTextures[NonUniformResourceIndex(texture)].SampleLevel(wrappingSampler, coordinates.sampleDirection, 0.0);
    return read;
}

// Normals are tangent space in their face's frame: x along the face's columns and y across them, both on the sphere's
// tangent plane; z is reconstructed. Without the layer, the sphere's normal.
float3 layerNormal(LayerRead read, LayerCoordinates coordinates, float3 modelNormal) {
    if (!read.resident) {
        return modelNormal;
    }

    float2 slope = read.value.rg * 2.0 - 1.0;
    float3 columnAxis = float3(faceColumns[coordinates.face]);
    float3 tangent = normalize(columnAxis - dot(columnAxis, modelNormal) * modelNormal);
    float3 bitangent = cross(tangent, modelNormal);
    float outward = sqrt(saturate(1.0 - dot(slope, slope)));
    return normalize(slope.x * tangent + slope.y * bitangent + outward * modelNormal);
}

// How many levels past the colour layer's finest texels a pixel magnifies it, from the angle a pixel spans at a unit
// `direction`: positive once a texel covers more than a pixel. Measured on the direction rather than on face
// coordinates, which jump at face edges.
float colorMagnification(LayerCoordinates coordinates, float3 direction) {
    float pixelAngle = max(length(ddx(direction)), length(ddy(direction)));
    float texelAngle = pi / 2.0 / float(max(coordinates.terrain.colorExtent, 1u));
    return log2(texelAngle / pixelAngle);
}
