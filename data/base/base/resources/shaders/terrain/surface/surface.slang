import debug.palette;
import draw;
import terrain.surface.fade;
import terrain.surface.blend;
import terrain.surface.hex_tiling;
import terrain.surface.normal_blend;
__exported import terrain.surface.surface_data;

// A plane takes part in the triplanar blend only where the normal leans toward its axis by more than this cosine: one
// plane on level ground mid-face, two along a face's edge, three at its corners.
static const float triplanarBias = 0.55;

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

// One sample of a surface: its colour (linear), its normal (body frame), its displacement (0 to 1) and the colours of
// the hexagonal cells it was drawn from.
struct PlaneSample {
    float3 color;
    float3 normal;
    float displacement;
    float3 cells;
};

// The body frame's axis `index`, built by comparison rather than read from a table.
float3 bodyAxis(int index) {
    return float3(index == 0 ? 1.0 : 0.0, index == 1 ? 1.0 : 0.0, index == 2 ? 1.0 : 0.0);
}

float2 planeCoordinates(float3 value, int plane) {
    return float2(value[(plane + 1) % 3], value[(plane + 2) % 3]);
}

// A variant of the surface projected on one plane, from the hexagonal cells around the pixel, its normal reoriented
// onto `normal` in the plane's frame.
PlaneSample samplePlane(SurfaceData *surface, HexAnchor anchor, uint variant, SurfacePoint point, int plane,
                        float3 normal) {
    float2 offset = planeCoordinates(point.offset, plane) / surface.size;
    float2 uvPerPixelX = planeCoordinates(point.offsetPerPixelX, plane) / surface.size;
    float2 uvPerPixelY = planeCoordinates(point.offsetPerPixelY, plane) / surface.size;
    HexCorners corners = hexCorners(anchor, offset / surface.hexCellSize);
    HexCellPlacement placements[3];
    float3 locations[3];
    float3 displacements = float3(0.0);

    [ForceUnroll]
    for (int corner = 0; corner < 3; corner++) {
        placements[corner] = hexCellPlacement(corners.cells[corner], uint(plane), surface.tilingSeed);
        float2 uv =
            mul(placements[corner].turn, corners.offsets[corner] * surface.hexCellSize) + placements[corner].shift;
        locations[corner] = float3(uv, float(variant));

        if (corners.weights[corner] > 0.0) {
            displacements[corner] = textureArrays[NonUniformResourceIndex(surface.displacementTexture)]
                                        .SampleGrad(tilingSampler,
                                                    locations[corner],
                                                    mul(placements[corner].turn, uvPerPixelX),
                                                    mul(placements[corner].turn, uvPerPixelY))
                                        .r;
        }
    }

    float3 shown = hexBlend(corners.weights, displacements);
    PlaneSample sample;
    sample.color = float3(0.0);
    sample.displacement = dot(displacements, shown);
    sample.cells = float3(0.0);
    float3 detail = float3(0.0);

    [ForceUnroll]
    for (int corner = 0; corner < 3; corner++) {
        if (shown[corner] <= 0.0) {
            continue;
        }

        float2x2 cellTurn = placements[corner].turn;
        float2 cellPerPixelX = mul(cellTurn, uvPerPixelX);
        float2 cellPerPixelY = mul(cellTurn, uvPerPixelY);
        sample.color += textureArrays[NonUniformResourceIndex(surface.colorTexture)]
                            .SampleGrad(tilingSampler, locations[corner], cellPerPixelX, cellPerPixelY)
                            .rgb *
                        shown[corner];
        float2 slope = textureArrays[NonUniformResourceIndex(surface.normalTexture)]
                               .SampleGrad(tilingSampler, locations[corner], cellPerPixelX, cellPerPixelY)
                               .rg *
                           2.0 -
                       1.0;
        // Green points to the image's top, against v; the slope turns back from texture to plane coordinates.
        float2 planeSlope = mul(transpose(cellTurn), float2(slope.x, -slope.y));
        detail += float3(planeSlope, sqrt(saturate(1.0 - dot(slope, slope)))) * shown[corner];
        sample.cells += hueColor(placements[corner].hue) * shown[corner];
    }

    float side = normal[plane] < 0.0 ? -1.0 : 1.0;
    float3 base = float3(planeCoordinates(normal, plane), abs(normal[plane]));
    float3 blended = reorientedNormal(base, normalize(detail));
    sample.normal = blended.x * bodyAxis((plane + 1) % 3) + blended.y * bodyAxis((plane + 2) % 3) +
                    blended.z * side * bodyAxis(plane);
    return sample;
}

// A variant of a surface at a pixel, with the plane weights it was drawn with and how much of it still shows.
struct SurfaceSample {
    float3 color;
    float3 normal;
    float displacement;
    float3 planeWeights;
    float3 cells;
    float fade;
};

// The colour, slope and displacement of one projection plane, added to their blends.
void addPlane(SurfaceData *surface, SurfaceAnchors *anchors, uint surfaceClass, uint variant, float weight,
              SurfacePoint point, int plane, float3 normal, inout ContrastBlend color, inout ContrastBlend slope,
              inout ContrastBlend displacement, inout ContrastBlend cells) {
    if (weight <= 0.0) {
        return;
    }

    PlaneSample sample = samplePlane(surface, anchors.planes[surfaceClass][plane], variant, point, plane, normal);
    color.add(sample.color - surface.averageColor, weight);
    slope.add(sample.normal - normal, weight);
    displacement.add(sample.displacement.xxx, weight);
    cells.add(sample.cells, weight);
}

// Projected on the planes the normal leans toward, blended so the surface keeps its contrast where they meet, and
// faded toward its average as the pixel's footprint grows to its whole repeat.
SurfaceSample sampleSurface(SurfaceData *surface, SurfaceAnchors *anchors, uint surfaceClass, uint variant,
                            SurfacePoint point, float3 normal) {
    SurfaceSample result;
    result.planeWeights = saturate(abs(normal) - triplanarBias);
    result.planeWeights /= result.planeWeights.x + result.planeWeights.y + result.planeWeights.z;
    result.fade = surfaceFadeOut(surface.size, point.footprint);
    result.color = surface.averageColor;
    result.normal = normal;
    result.displacement = 0.0;
    result.cells = float3(0.0);

    if (result.fade <= 0.0) {
        return result;
    }

    ContrastBlend color = contrastBlend();
    ContrastBlend slope = contrastBlend();
    ContrastBlend displacement = contrastBlend();
    ContrastBlend cells = contrastBlend();

    for (int plane = 0; plane < triplanarPlanes; plane++) {
        addPlane(surface,
                 anchors,
                 surfaceClass,
                 variant,
                 result.planeWeights[plane],
                 point,
                 plane,
                 normal,
                 color,
                 slope,
                 displacement,
                 cells);
    }

    result.color = max(surface.averageColor + color.deviation() * result.fade, 0.0);
    result.normal = normalize(normal + slope.deviation() * result.fade);
    result.displacement = displacement.average().x;
    result.cells = cells.average();
    return result;
}
