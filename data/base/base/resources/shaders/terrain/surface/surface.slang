import draw;
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

// The colour, slope and displacement of one projection plane at one level, added to their blends.
void addPlane(SurfaceData *surface, SurfaceAnchors *anchors, uint surfaceClass, float weight, SurfacePoint point,
              int plane, int level, float3 normal, inout ContrastBlend color, inout ContrastBlend slope,
              inout ContrastBlend displacement) {
    if (weight <= 0.0) {
        return;
    }

    PlaneSample sample = samplePlane(surface, anchors.repeats[surfaceClass][plane][level], point, plane, level, normal);
    color.add(sample.color - surface.averageColor, weight);
    slope.add(sample.normal - normal, weight);
    displacement.add(sample.displacement.xxx, weight);
}

// Projected on the two planes the normal leans toward most, at the two levels around the pixel's own, blended so the
// surface keeps its contrast where they meet.
SurfaceSample sampleSurface(SurfaceData *surface, SurfaceAnchors *anchors, uint surfaceClass, SurfacePoint point,
                            float3 normal) {
    float3 lean = abs(normal);
    int major = lean.x >= lean.y && lean.x >= lean.z ? 0 : (lean.y >= lean.z ? 1 : 2);
    int next = (major + 1) % 3;
    int last = (major + 2) % 3;
    int median = lean[next] >= lean[last] ? next : last;
    SurfaceSample result;
    result.planeWeights = float3(0.0);
    result.planeWeights[major] = lean[major] - biplanarBias;
    result.planeWeights[median] = saturate(lean[median] - biplanarBias);
    result.planeWeights /= result.planeWeights[major] + result.planeWeights[median];
    ContrastBlend color = contrastBlend();
    ContrastBlend slope = contrastBlend();
    ContrastBlend displacement = contrastBlend();

    int2 planes = int2(major, median);
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
            int plane = planes[index];
            addPlane(surface,
                     anchors,
                     surfaceClass,
                     result.planeWeights[plane] * levelWeights[step],
                     point,
                     plane,
                     finer + step,
                     normal,
                     color,
                     slope,
                     displacement);
        }
    }

    result.color = max(surface.averageColor + color.deviation(), 0.0);
    result.normal = normalize(normal + slope.deviation());
    result.displacement = displacement.average().x;
    return result;
}
