import draw;
import terrain.tile;
import terrain.surface.blend;
import terrain.surface.normal_blend;
__exported import terrain.surface.surface_data;

// A plane weighs the normal's component along its axis beyond 1/√3. A component above it is never the smallest, so the
// second plane weighs nothing wherever it could swap with the third, and only one plane is sampled mid-face.
static const float biplanarBias = 0.57735027;

// A surface's repeat keeps its size until it spans fewer than this many pixels, then grows `surfaceLevelGrowth` times
// each time the pixel's footprint grows this ratio, so it shrinks slowly on screen and never shows a dense grid.
static const float surfaceRepeatPixels = 512.0;
static const float surfaceLevelDistanceRatio = 3.4;

// Turns a 32-bit hash into [0, 1).
static const float hashToUnit = 1.0 / 4294967296.0;

// A pixel's offset from the camera in the body's frame (m), how it changes across the screen, and the ground it covers
// (m). Its derivatives are taken where every pixel of a quad still runs, before any surface is sampled.
struct SurfacePoint {
    float3 offset;
    float3 offsetPerPixelX;
    float3 offsetPerPixelY;
    float footprint;
};

SurfacePoint surfacePoint(float3 worldOffset, ObjectData object) {
    float3x3 worldFromModel = (float3x3)object.worldFromModel;
    float radius = length(worldFromModel[0]);
    SurfacePoint point;
    point.offset = mul(transpose(worldFromModel), worldOffset) / radius;
    point.offsetPerPixelX = ddx(point.offset);
    point.offsetPerPixelY = ddy(point.offset);
    point.footprint = min(length(point.offsetPerPixelX), length(point.offsetPerPixelY));
    return point;
}

// One sample of a surface: its colour (linear), its normal (body frame) and its displacement (0 to 1).
struct PlaneSample {
    float3 color;
    float3 normal;
    float displacement;
};

// The body frame's axis `index`, built by comparison rather than read from a table.
float3 bodyAxis(int index) {
    return float3(index == 0 ? 1.0 : 0.0, index == 1 ? 1.0 : 0.0, index == 2 ? 1.0 : 0.0);
}

float2 planeCoordinates(float3 value, int plane) {
    return float2(value[(plane + 1) % 3], value[(plane + 2) % 3]);
}

// The continuous level a pixel covering `footprint` meters reads a surface repeating every `size` meters at.
float surfaceLevel(float size, float footprint) {
    float level = log(footprint * surfaceRepeatPixels / size) / log(surfaceLevelDistanceRatio);
    return clamp(level, 0.0, float(surfaceLevels - 1));
}

// The seeded shift (repeats) of a surface's repeat at one level on one plane, so its levels and planes never line up.
float2 repeatShift(int level, int plane, uint seed) {
    uint3 hash = pcg3d(uint3(uint(level), uint(plane), seed));
    return float2(hash.xy) * hashToUnit;
}

// A surface's relief `detail`, in the frame of projection plane `plane`, reoriented onto `normal` in the body's frame.
float3 planeNormal(float3 detail, float3 normal, int plane) {
    float side = normal[plane] < 0.0 ? -1.0 : 1.0;
    float3 base = float3(planeCoordinates(normal, plane), abs(normal[plane]));
    float3 blended = reorientedNormal(base, normalize(detail));
    return blended.x * bodyAxis((plane + 1) % 3) + blended.y * bodyAxis((plane + 2) % 3) +
           blended.z * side * bodyAxis(plane);
}

// The surface projected on one plane at one level, its repeat shifted by its seeded offset.
PlaneSample samplePlane(SurfaceData *surface, float2 repeatPlace, SurfacePoint point, int plane, int level,
                        float3 normal) {
    float repeat = surface.size * pow(surfaceLevelGrowth, float(level));
    float2 uv =
        repeatPlace + planeCoordinates(point.offset, plane) / repeat + repeatShift(level, plane, surface.tilingSeed);
    float3 location = float3(uv, 0.0);
    float2 uvPerPixelX = planeCoordinates(point.offsetPerPixelX, plane) / repeat;
    float2 uvPerPixelY = planeCoordinates(point.offsetPerPixelY, plane) / repeat;
    PlaneSample sample;
    sample.color = textureArrays[NonUniformResourceIndex(surface.colorTexture)]
                       .SampleGrad(tilingSampler, location, uvPerPixelX, uvPerPixelY)
                       .rgb;
    sample.displacement = textureArrays[NonUniformResourceIndex(surface.displacementTexture)]
                              .SampleGrad(tilingSampler, location, uvPerPixelX, uvPerPixelY)
                              .r;
    float2 slope = textureArrays[NonUniformResourceIndex(surface.normalTexture)]
                           .SampleGrad(tilingSampler, location, uvPerPixelX, uvPerPixelY)
                           .rg *
                       2.0 -
                   1.0;
    // Green points to the image's top, against v.
    float3 detail = float3(slope.x, -slope.y, sqrt(saturate(1.0 - dot(slope, slope))));
    sample.normal = planeNormal(detail, normal, plane);
    return sample;
}

// The two planes a normal leans toward most, and their weights, summing to 1.
struct ProjectionPlanes {
    int2 planes;
    float2 weights;
};

ProjectionPlanes projectionPlanes(float3 normal) {
    float3 lean = abs(normal);
    int major = lean.x >= lean.y && lean.x >= lean.z ? 0 : (lean.y >= lean.z ? 1 : 2);
    int next = (major + 1) % 3;
    int last = (major + 2) % 3;
    int median = lean[next] >= lean[last] ? next : last;
    ProjectionPlanes projection;
    projection.planes = int2(major, median);
    projection.weights = float2(lean[major] - biplanarBias, saturate(lean[median] - biplanarBias));
    projection.weights /= projection.weights.x + projection.weights.y;
    return projection;
}

// A surface's displacement texture, its width in texels and its number of mip levels.
struct DisplacementTexture {
    Texture2DArray texture;
    float width;
    uint levels;
};

DisplacementTexture displacementTexture(SurfaceData *surface) {
    DisplacementTexture displacement;
    displacement.texture = textureArrays[NonUniformResourceIndex(surface.displacementTexture)];
    uint width;
    uint height;
    uint layers;
    displacement.texture.GetDimensions(0, width, height, layers, displacement.levels);
    displacement.width = float(width);
    return displacement;
}

// The surface's average displacement: its smallest mip, a single texel.
float meanDisplacement(DisplacementTexture displacement) {
    return displacement.texture.SampleLevel(tilingSampler, float3(0.5, 0.5, 0.0), float(displacement.levels - 1)).r;
}

// A surface at a pixel, with the plane weights it was drawn with.
struct SurfaceSample {
    float3 color;
    float3 normal;
    float displacement;
    float3 planeWeights;
};

// A surface left unsampled, where it has no weight: its average colour on the ground's own normal.
SurfaceSample unsampledSurface(SurfaceData *surface, float3 normal) {
    SurfaceSample sample;
    sample.color = surface.averageColor;
    sample.normal = normal;
    sample.displacement = 0.0;
    sample.planeWeights = float3(0.0);
    return sample;
}

// The colour, slope and displacement of one projection plane at one level, added to their blends around the
// surface's averages.
void addPlane(SurfaceData *surface, SurfaceAnchors *anchors, uint surfaceClass, float weight, SurfacePoint point,
              int plane, int level, float3 normal, float meanHeight, inout ContrastBlend color,
              inout ContrastBlend slope, inout ContrastBlend displacement) {
    if (weight <= 0.0) {
        return;
    }

    PlaneSample sample = samplePlane(surface, anchors.repeats[surfaceClass][plane][level], point, plane, level, normal);
    color.add(sample.color - surface.averageColor, weight);
    slope.add(sample.normal - normal, weight);
    displacement.add((sample.displacement - meanHeight).xxx, weight);
}

// Projected on the two planes the normal leans toward most, at the two levels around the pixel's own, blended so the
// surface keeps its contrast where they meet.
SurfaceSample sampleSurface(SurfaceData *surface, SurfaceAnchors *anchors, uint surfaceClass, SurfacePoint point,
                            float3 normal) {
    ProjectionPlanes projection = projectionPlanes(normal);
    float meanHeight = meanDisplacement(displacementTexture(surface));
    SurfaceSample result;
    result.planeWeights = float3(0.0);
    result.planeWeights[projection.planes.x] = projection.weights.x;
    result.planeWeights[projection.planes.y] = projection.weights.y;
    ContrastBlend color = contrastBlend();
    ContrastBlend slope = contrastBlend();
    ContrastBlend displacement = contrastBlend();
    float level = surfaceLevel(surface.size, point.footprint);
    int finer = int(level);
    float2 levelWeights = float2(1.0 - (level - float(finer)), level - float(finer));

    [ForceUnroll]
    for (int step = 0; step < 2; step++) {
        if (levelWeights[step] <= 0.0) {
            continue;
        }

        [ForceUnroll]
        for (int index = 0; index < 2; index++) {
            addPlane(surface,
                     anchors,
                     surfaceClass,
                     projection.weights[index] * levelWeights[step],
                     point,
                     projection.planes[index],
                     finer + step,
                     normal,
                     meanHeight,
                     color,
                     slope,
                     displacement);
        }
    }

    result.color = max(surface.averageColor + color.deviation(), 0.0);
    result.normal = normalize(normal + slope.deviation());
    result.displacement = meanHeight + displacement.deviation().x;
    return result;
}

// A surface's displacement (0 to 1) under a vertex `offset` (m, body frame) from the camera, at its true-size repeat
// filtered to the `vertexSpacing` (m) between vertices, and how far it moves the ground from its average (m). It fades
// out as the pixels there, covering `pixelFootprint` (m), move on to the next repeat size.
struct ReliefSample {
    float displacement;
    float meters;
};

ReliefSample noRelief() {
    ReliefSample sample;
    sample.displacement = 0.0;
    sample.meters = 0.0;
    return sample;
}

ReliefSample surfaceReliefSample(SurfaceData *surface, SurfaceAnchors *anchors, uint surfaceClass, float3 offset,
                                 float3 normal, float pixelFootprint, float vertexSpacing) {
    DisplacementTexture displacement = displacementTexture(surface);
    float meanHeight = meanDisplacement(displacement);
    float strength = saturate(1.0 - surfaceLevel(surface.size, pixelFootprint));
    ReliefSample sample;
    sample.displacement = meanHeight;
    sample.meters = 0.0;

    if (strength <= 0.0) {
        return sample;
    }

    float mip = max(log2(vertexSpacing * displacement.width / surface.size), 0.0);
    ProjectionPlanes projection = projectionPlanes(normal);
    ContrastBlend blend = contrastBlend();

    [ForceUnroll]
    for (int index = 0; index < 2; index++) {
        int plane = projection.planes[index];

        if (projection.weights[index] <= 0.0) {
            continue;
        }

        float2 uv = anchors.repeats[surfaceClass][plane][0] + planeCoordinates(offset, plane) / surface.size +
                    repeatShift(0, plane, surface.tilingSeed);
        float height = displacement.texture.SampleLevel(tilingSampler, float3(uv, 0.0), mip).r;
        blend.add((height - meanHeight).xxx, projection.weights[index]);
    }

    float deviation = blend.deviation().x;
    sample.displacement = meanHeight + deviation;
    sample.meters = deviation * (surface.displacementHigh - surface.displacementLow) * strength;
    return sample;
}
