import light.hapke;
import light.lambert;
import light.star;
import terrain.layers;
import terrain.output;
import terrain.texture;

[shader("vertex")]
TileVertex vertexMain(VertexInput input) {
    return terrainVertex(input);
}

struct HapkeData {
    float3 baseColor;
    uint emissiveTexture;
    float emissiveLuminance;
    uint padding[3];
};

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

// Where the tile has no page of either Hapke layer, it is shaded with Lambert.
[shader("fragment")]
FragmentOutput fragmentMain(TileVertex input, uint primitive: SV_PrimitiveID) {
    HapkeData material = currentMaterial<HapkeData>();
    float3 modelNormal = normalize(input.draw.modelNormal);
    LayerCoordinates coordinates = layerCoordinates(input.tile, input.faceOffset);
    LayerRead color = readLayer(coordinates, colorLayer);
    LayerRead medium = readLayer(coordinates, mediumLayer);
    LayerRead opposition = readLayer(coordinates, oppositionLayer);
    float3 normal = layerNormal(readLayer(coordinates, normalLayer), coordinates.tile, modelNormal);
    float3 worldNormal = normalize(mul(currentObject().worldFromModel, float4(normal, 0.0)).xyz);
    float3 geometricNormal = normalize(input.draw.normal);
    float3 albedo = color.resident ? color.value.rgb : material.baseColor;
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
