import material.material;

struct ScaleVaryings {
    SurfaceVaryings surface;
    float heightMeters;
};

[shader("vertex")]
ScaleVaryings vertexMain<G : Geometry>(VertexInput input) {
    TileVertex tile = G.transform(input);
    ScaleVaryings output;
    output.surface = tile.surface;
    output.heightMeters = tile.heightMeters;
    return output;
}

// Each power of ten of meters, from 1 m to 10 km, draws in its own color: range rings bright, height contours darker.
static const uint decades = 5;
static const float smallestSpacingMeters = 1.0;
static const float3 decadeColors[decades] = {
    float3(1.0, 0.25, 0.25),
    float3(1.0, 0.6, 0.1),
    float3(1.0, 1.0, 0.2),
    float3(0.3, 1.0, 0.3),
    float3(0.2, 0.9, 1.0),
};
static const float contourShade = 0.45;
static const float lineWidthPixels = 1.0;
// A spacing fades in as its lines grow from this far apart on screen to twice that.
static const float fadeStartPixels = 8.0;
static const float flatShade = 0.8;
static const float steepShade = 0.25;
static const float baseShade = 0.5;

// How much of a line through each multiple of `spacing` covers this pixel, faded out where the lines crowd together.
float lineStrength(float meters, float spacing) {
    float coordinate = meters / spacing;
    float perPixel = fwidth(coordinate);
    float coverage = 1.0 - saturate(abs(coordinate - round(coordinate)) / perPixel / lineWidthPixels);
    float spacingPixels = 1.0 / perPixel;
    float fade = saturate((spacingPixels - fadeStartPixels) / fadeStartPixels);
    return coverage * fade;
}

[shader("fragment")]
SurfaceOutput fragmentMain(ScaleVaryings input) {
    ObjectData object = currentObject();
    float3 bodyCenter = float3(object.worldFromModel[0][3], object.worldFromModel[1][3], object.worldFromModel[2][3]);
    float3 up = normalize(-bodyCenter);
    float3 position = input.surface.worldPosition;
    float3 normal = normalize(cross(ddy(position), ddx(position)));
    normal = dot(normal, position) > 0.0 ? -normal : normal;
    float3 color = baseShade * lerp(steepShade, flatShade, saturate(dot(normal, up)));
    float rangeMeters = length(position - dot(position, up) * up);
    float spacing = smallestSpacingMeters;

    for (uint decade = 0; decade < decades; decade++) {
        color = lerp(color, decadeColors[decade] * contourShade, lineStrength(input.heightMeters, spacing));
        color = lerp(color, decadeColors[decade], lineStrength(rangeMeters, spacing));
        spacing *= 10.0;
    }

    return surfaceOutput(float4(color, 1.0), input.surface);
}
