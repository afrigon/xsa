import material.light.hapke;
import material.light.star;
import material.material;
import material.texture;

[shader("vertex")]
SurfaceVaryings vertexMain<G : Geometry>(VertexInput input) {
    return G.transform(input).surface;
}

struct HapkeData {
    float3 baseColor;
    uint colorTexture;
    uint normalTexture;
    uint emissiveTexture;
    float emissiveLuminance;
    uint mediumTexture;
    uint oppositionTexture;
    uint padding[3];
};

// Hapke 2012's parameters at this point: w from the calibrated color map, the rest from the medium and opposition
// textures, stored as published (θ̄ in degrees).
HapkeParameters hapkeParameters(HapkeData material, TextureCoordinates coordinates, float3 diffusiveReflectance) {
    float4 medium = sampleTexture(material.mediumTexture, coordinates);
    float4 opposition = sampleTexture(material.oppositionTexture, coordinates);
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

// cd/m² the surface reflects from one star.
float3 reflectedLuminance(StarData star, HapkeParameters parameters, SurfaceVaryings input, float3 worldNormal) {
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

[shader("fragment")]
SurfaceOutput fragmentMain(SurfaceVaryings input) {
    HapkeData material = currentMaterial<HapkeData>();
    float3 modelNormal = normalize(input.modelNormal);
    TextureCoordinates coordinates = equirectangularCoordinates(modelNormal);
    float3 normal = mappedNormal(material.normalTexture, modelNormal, coordinates);
    float3 worldNormal = normalize(mul(currentObject().worldFromModel, float4(normal, 0.0)).xyz);
    float3 albedo = mappedColor(material.colorTexture, material.baseColor, coordinates);
    HapkeParameters parameters = hapkeParameters(material, coordinates, albedo);
    float3 luminance = starlightLuminance(albedo);

    for (uint index = 0; index < constants.frame.starCount; index++) {
        luminance += reflectedLuminance(constants.frame.stars[index], parameters, input, worldNormal);
    }

    luminance += mappedEmission(material.emissiveTexture, material.emissiveLuminance, coordinates);
    return surfaceOutput(float4(luminance * constants.frame.exposure, 1.0), input);
}
