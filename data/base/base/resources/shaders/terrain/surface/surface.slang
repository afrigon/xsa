import draw;
import terrain.surface.blend;
import terrain.surface.normal_blend;
__exported import terrain.surface.surface_data;

// A plane takes part in the triplanar blend only where the normal leans toward its axis by more than this cosine: one
// plane on level ground mid-face, two along a face's edge, three at its corners.
static const float triplanarBias = 0.55;

// A pixel's offset from the camera in the body's frame (m), and how it changes across the screen. Its derivatives are
// taken where every pixel of a quad still runs, before any surface is sampled.
struct SurfacePoint {
    float3 offset;
    float3 offsetPerPixelX;
    float3 offsetPerPixelY;
};

SurfacePoint surfacePoint(float3 worldOffset, ObjectData object) {
    float3x3 worldFromModel = (float3x3)object.worldFromModel;
    float radius = length(worldFromModel[0]);
    SurfacePoint point;
    point.offset = mul(transpose(worldFromModel), worldOffset) / radius;
    point.offsetPerPixelX = ddx(point.offset);
    point.offsetPerPixelY = ddy(point.offset);
    return point;
}

// One sample of a surface: its colour (linear), its normal (body frame) and its displacement (0 to 1).
struct ScaleSample {
    float3 color;
    float3 normal;
    float displacement;
};

float2 planeCoordinates(float3 value, int plane) {
    return float2(value[(plane + 1) % 3], value[(plane + 2) % 3]);
}

// A variant of the surface on one projection plane at one scale, its normal reoriented onto `normal` in the plane's
// frame.
ScaleSample sampleScale(SurfaceData surface, float2 anchor, uint variant, int scale, SurfacePoint point, int plane,
                        float3 normal) {
    float repeat = surface.size * exp2(float(scale));
    float2 rotation = surface.scaleRotations[scale];
    float2x2 turn = float2x2(rotation.x, -rotation.y, rotation.y, rotation.x);
    float2 uv = anchor + mul(turn, planeCoordinates(point.offset, plane)) / repeat;
    float2 uvPerPixelX = mul(turn, planeCoordinates(point.offsetPerPixelX, plane)) / repeat;
    float2 uvPerPixelY = mul(turn, planeCoordinates(point.offsetPerPixelY, plane)) / repeat;
    float3 location = float3(uv, float(variant));

    ScaleSample sample;
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
    // Green points to the image's top, against v; the slope turns back from texture to plane coordinates.
    float2 planeSlope = mul(transpose(turn), float2(slope.x, -slope.y));
    float3 detail = float3(planeSlope, sqrt(saturate(1.0 - dot(slope, slope))));

    float side = normal[plane] < 0.0 ? -1.0 : 1.0;
    float3 base = float3(planeCoordinates(normal, plane), abs(normal[plane]));
    float3 blended = reorientedNormal(base, detail);
    float3 axes[3] = { float3(1.0, 0.0, 0.0), float3(0.0, 1.0, 0.0), float3(0.0, 0.0, 1.0) };
    sample.normal =
        blended.x * axes[(plane + 1) % 3] + blended.y * axes[(plane + 2) % 3] + blended.z * side * axes[plane];
    return sample;
}

// A variant of a surface at a pixel, with the scale level and plane weights it was drawn with.
struct SurfaceSample {
    float3 color;
    float3 normal;
    float displacement;
    float level;
    float3 planeWeights;
};

// Projected on the planes the normal leans toward, at the two scales of the ladder around the pixel's footprint on
// the ground, blended so the surface keeps its contrast where they meet.
SurfaceSample sampleSurface(SurfaceData surface, SurfaceAnchors *anchors, uint surfaceClass, uint variant,
                            SurfacePoint point, float3 normal) {
    SurfaceSample result;
    result.planeWeights = saturate(abs(normal) - triplanarBias);
    result.planeWeights /= result.planeWeights.x + result.planeWeights.y + result.planeWeights.z;

    float footprint = min(length(point.offsetPerPixelX), length(point.offsetPerPixelY));
    result.level = clamp(log2(footprint * surface.texels / surface.size), 0.0, float(surface.scaleCount - 1));
    int finer = int(result.level);
    int coarser = min(finer + 1, int(surface.scaleCount) - 1);
    float scaleWeights[2] = { 1.0 - (result.level - float(finer)), result.level - float(finer) };
    int scales[2] = { finer, coarser };

    ContrastBlend color = contrastBlend();
    ContrastBlend slope = contrastBlend();
    ContrastBlend displacement = contrastBlend();

    for (int plane = 0; plane < triplanarPlanes; plane++) {
        if (result.planeWeights[plane] <= 0.0) {
            continue;
        }

        for (int index = 0; index < 2; index++) {
            float weight = result.planeWeights[plane] * scaleWeights[index];

            if (weight <= 0.0) {
                continue;
            }

            int scale = scales[index];
            ScaleSample sample =
                sampleScale(surface, anchors.planes[surfaceClass][scale][plane], variant, scale, point, plane, normal);
            color.add(sample.color - surface.averageColor, weight);
            slope.add(sample.normal - normal, weight);
            displacement.add(sample.displacement.xxx, weight);
        }
    }

    result.color = max(surface.averageColor + color.deviation(), 0.0);
    result.normal = normalize(normal + slope.deviation());
    result.displacement = displacement.average().x;
    return result;
}
