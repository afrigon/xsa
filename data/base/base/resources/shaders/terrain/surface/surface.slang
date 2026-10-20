import draw;
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

// A surface's colour (linear) and normal (body frame) at a pixel.
struct SurfaceSample {
    float3 color;
    float3 normal;
};

float2 planeCoordinates(float3 value, int plane) {
    return float2(value[(plane + 1) % 3], value[(plane + 2) % 3]);
}

// The surface on one projection plane at one scale, its normal reoriented onto `normal` in the plane's frame.
SurfaceSample sampleScale(SurfaceData surface, float2 anchor, int scale, SurfacePoint point, int plane, float3 normal) {
    float repeat = surface.size * exp2(float(scale));
    float2 rotation = surface.scaleRotations[scale];
    float2x2 turn = float2x2(rotation.x, -rotation.y, rotation.y, rotation.x);
    float2 uv = anchor + mul(turn, planeCoordinates(point.offset, plane)) / repeat;
    float2 uvPerPixelX = mul(turn, planeCoordinates(point.offsetPerPixelX, plane)) / repeat;
    float2 uvPerPixelY = mul(turn, planeCoordinates(point.offsetPerPixelY, plane)) / repeat;
    float3 location = float3(uv, 0.0);

    SurfaceSample sample;
    sample.color = textureArrays[NonUniformResourceIndex(surface.colorTexture)]
                       .SampleGrad(tilingSampler, location, uvPerPixelX, uvPerPixelY)
                       .rgb;
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

// A surface at a pixel: projected on the planes the normal leans toward, at the two scales of its ladder around the
// pixel's footprint on the ground.
SurfaceSample sampleSurface(SurfaceData surface, SurfaceAnchors *anchors, uint surfaceClass, SurfacePoint point,
                            float3 normal) {
    float3 planeWeights = saturate(abs(normal) - triplanarBias);
    planeWeights /= planeWeights.x + planeWeights.y + planeWeights.z;

    float footprint = min(length(point.offsetPerPixelX), length(point.offsetPerPixelY));
    float level = clamp(log2(footprint * surface.texels / surface.size), 0.0, float(surface.scaleCount - 1));
    int finer = int(level);
    int coarser = min(finer + 1, int(surface.scaleCount) - 1);
    float scaleWeights[2] = { 1.0 - (level - float(finer)), level - float(finer) };
    int scales[2] = { finer, coarser };

    SurfaceSample sum;
    sum.color = float3(0.0);
    sum.normal = float3(0.0);

    for (int plane = 0; plane < triplanarPlanes; plane++) {
        if (planeWeights[plane] <= 0.0) {
            continue;
        }

        for (int index = 0; index < 2; index++) {
            float weight = planeWeights[plane] * scaleWeights[index];

            if (weight <= 0.0) {
                continue;
            }

            int scale = scales[index];
            SurfaceSample sample =
                sampleScale(surface, anchors.planes[surfaceClass][scale][plane], scale, point, plane, normal);
            sum.color += sample.color * weight;
            sum.normal += sample.normal * weight;
        }
    }

    sum.normal = normalize(sum.normal);
    return sum;
}
