import light.hapke;
import light.lambert;
import light.star;
import terrain.layers;
import terrain.output;
import terrain.surface.fade;
import terrain.surface.range;
import terrain.surface.relief;
import terrain.surface.surface;
import terrain.surface.blend;
import terrain.surface.classes;
import terrain.surface.debug_view;
import terrain.texture;

// The terrain's varyings and the ground's shape, which picks the class of its surfaces.
struct HapkeVaryings {
    TerrainVaryings terrain;
    float curvature;
    float relativeHeight;
};

// Mirrors `HapkeData` in the renderer. Its surfaces are read only once every class's arrays are loaded.
struct HapkeData {
    float3 baseColor;
    uint emissiveTexture;
    float emissiveLuminance;
    uint surfacesLoaded;
    uint padding[2];
    BiomeData biome;
    SurfaceData surfaces[surfaceClasses];
};

[shader("vertex")]
HapkeVaryings vertexMain(VertexInput input) {
    HapkeData *material = currentMaterialAddress<HapkeData>();
    SurfaceRelief relief;
    relief.biome = &material.biome;
    relief.surfaces = &material.surfaces[0];
    relief.loaded = material.surfacesLoaded != 0;
    TileVertex vertex = terrainVertex(input, relief);
    HapkeVaryings output;
    output.terrain = terrainVaryings(vertex);
    output.curvature = vertex.curvature;
    output.relativeHeight = vertex.relativeHeight;
    return output;
}

// Hapke 2012's parameters at this point: w from the calibrated color layer, the rest from the medium and opposition
// layers, stored as published (θ̄ in degrees).
HapkeParameters hapkeParameters(float4 medium, float4 opposition, float3 diffusiveReflectance) {
    HapkeParameters parameters;
    parameters.singleScatteringAlbedo = singleScatteringAlbedo(diffusiveReflectance);
    parameters.lobeWidth = medium.r;
    parameters.lobeBalance = medium.g;
    parameters.roughness = radians(medium.b);
    parameters.porosity = medium.a;
    parameters.shadowHidingAmplitude = opposition.r;
    parameters.shadowHidingWidth = opposition.g;
    parameters.backscatterAmplitude = opposition.b;
    parameters.backscatterWidth = opposition.a;
    return parameters;
}

// cd/m² the terrain reflects from one star.
float3 reflectedLuminance(StarData star, HapkeParameters parameters, DrawVaryings input, float3 worldNormal) {
    float cosine = mappedCosine(star, input.worldPosition, normalize(input.normal), worldNormal);

    if (cosine <= 0.0) {
        return float3(0.0);
    }

    float3 toCamera = normalize(-input.worldPosition);
    float3 toStar = normalize(star.positionAndRadius.xyz - input.worldPosition);
    HapkeGeometry geometry;
    geometry.incidenceCosine = clamp(dot(toStar, worldNormal), 1e-4, 1.0);
    geometry.emissionCosine = clamp(dot(toCamera, worldNormal), 1e-4, 1.0);
    geometry.phaseAngle = acos(clamp(dot(toStar, toCamera), -1.0, 1.0));
    return hapkeReflectance(parameters, geometry) * starIlluminance(star, input.worldPosition);
}

// The ground up close: the flat and slope surfaces weighed at the pixel and blended by height where both show,
// brightening or darkening the colour layer and adding their relief to the normal layer's, fading in as the colour
// layer is magnified past its texels. `debugColor` is what the surface debug view shows.
void addSurfaces(HapkeData *material, LayerCoordinates coordinates, SurfacePoint point, HapkeVaryings shape, float3 up,
                 inout float3 albedo, inout float3 normal, out float3 debugColor) {
    debugColor = float3(0.0);

    if (!shadesSurfaces() || material.surfacesLoaded == 0) {
        return;
    }

    float fade = surfaceFade(coordinates);

    if (fade <= 0.0) {
        return;
    }

    SurfaceAnchors *anchors = surfaceAnchors();
    float2 weights =
        surfaceClassWeights(material.biome, anchors, point.offset, normal, up, shape.curvature, shape.relativeHeight);
    SurfaceData *flatSurface = &material.surfaces[uint(SurfaceClass.Flat)];
    SurfaceData *slopeSurface = &material.surfaces[uint(SurfaceClass.Slope)];
    SurfaceSample flat = unsampledSurface(flatSurface, normal);
    SurfaceSample sloped = unsampledSurface(slopeSurface, normal);

    if (weights.x > 0.0) {
        flat = sampleSurface(flatSurface, anchors, uint(SurfaceClass.Flat), point, normal);
    }

    if (weights.y > 0.0) {
        sloped = sampleSurface(slopeSurface, anchors, uint(SurfaceClass.Slope), point, normal);
    }

    if (weights.x > 0.0 && weights.y > 0.0) {
        weights = heightBlend(weights, float2(flat.displacement, sloped.displacement), material.biome.blendSharpness);
    }

    float3 color =
        flat.color / flatSurface.averageColor * weights.x + sloped.color / slopeSurface.averageColor * weights.y;
    float3 relief = normalize(flat.normal * weights.x + sloped.normal * weights.y);
    albedo *= lerp(float3(1.0), color, fade);
    normal = normalize(lerp(normal, relief, fade));

    switch (surfaceDebugView) {
    case uint(SurfaceDebugView.Classes):
        debugColor = float3(weights, 0.0);
        break;
    case uint(SurfaceDebugView.Fade):
        debugColor = fade.xxx;
        break;
    case uint(SurfaceDebugView.Planes):
        debugColor = weights.x >= weights.y ? flat.planeWeights : sloped.planeWeights;
        break;
    default:
        break;
    }
}

// Where the tile has no page of either Hapke layer, it is shaded with Lambert.
[shader("fragment")]
FragmentOutput fragmentMain(HapkeVaryings varyings, uint primitive: SV_PrimitiveID) {
    TerrainVaryings input = varyings.terrain;
    HapkeData *material = currentMaterialAddress<HapkeData>();
    float3 modelNormal = normalize(input.draw.modelNormal);
    LayerCoordinates coordinates = layerCoordinates(input.tile, input.faceOffset);
    LayerRead color = readLayer(coordinates, colorLayer);
    LayerRead medium = readLayer(coordinates, mediumLayer);
    LayerRead opposition = readLayer(coordinates, oppositionLayer);
    float3 normal = layerNormal(readLayer(coordinates, normalLayer), coordinates.tile, modelNormal);
    float3 albedo = color.resident ? color.value.rgb : material.baseColor;
    float3 surfaceDebugColor;
    addSurfaces(material,
                coordinates,
                surfacePoint(input.draw.worldPosition, currentObject()),
                varyings,
                modelNormal,
                albedo,
                normal,
                surfaceDebugColor);

    if (showsSurfaceDebugView()) {
        return fragmentOutput(float4(surfaceDebugColor, 1.0), input.draw);
    }

    float3 worldNormal = normalize(mul(currentObject().worldFromModel, float4(normal, 0.0)).xyz);
    float3 geometricNormal = normalize(input.draw.normal);
    float3 luminance = starlightLuminance(albedo);

    if (medium.resident && opposition.resident) {
        HapkeParameters parameters = hapkeParameters(medium.value, opposition.value, albedo);

        for (uint index = 0; index < constants.frame.starCount; index++) {
            luminance += reflectedLuminance(constants.frame.stars[index], parameters, input.draw, worldNormal);
        }
    } else {
        for (uint index = 0; index < constants.frame.starCount; index++) {
            StarData star = constants.frame.stars[index];
            float cosine = mappedCosine(star, input.draw.worldPosition, geometricNormal, worldNormal);
            luminance += lambertLuminance(albedo, starIlluminance(star, input.draw.worldPosition), cosine);
        }
    }

    TextureCoordinates emissionCoordinates = equirectangularCoordinates(modelNormal);
    luminance += mappedEmission(material.emissiveTexture, material.emissiveLuminance, emissionCoordinates);
    MaterialSample sample;
    sample.albedo = albedo;
    sample.roughness = medium.resident ? radians(medium.value.b) : 0.0;
    return terrainOutput(luminance, sample, input, primitive);
}
