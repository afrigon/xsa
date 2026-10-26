import light.bounce;
import light.hapke;
import light.lambert;
import light.star;
import mesh.output;

// Mirrors `HapkeData` in the renderer. Radians for the roughness.
struct HapkeData {
    float3 baseColor;
    float lobeWidth;
    float lobeBalance;
    float macroscopicRoughness;
    float porosity;
    float shadowHidingAmplitude;
    float shadowHidingWidth;
    float backscatterAmplitude;
    float backscatterWidth;
    uint padding;
};

[shader("vertex")]
MeshVaryings vertexMain(VertexInput input) {
    return meshVertex(input);
}

HapkeParameters hapkeParameters(HapkeData material, float3 diffusiveReflectance) {
    HapkeParameters parameters;
    parameters.singleScatteringAlbedo = singleScatteringAlbedo(diffusiveReflectance);
    parameters.lobeWidth = material.lobeWidth;
    parameters.lobeBalance = material.lobeBalance;
    parameters.roughness = material.macroscopicRoughness;
    parameters.porosity = material.porosity;
    parameters.shadowHidingAmplitude = material.shadowHidingAmplitude;
    parameters.shadowHidingWidth = material.shadowHidingWidth;
    parameters.backscatterAmplitude = material.backscatterAmplitude;
    parameters.backscatterWidth = material.backscatterWidth;
    return parameters;
}

// cd/m² the mesh reflects from one star.
float3 reflectedLuminance(StarData star, HapkeParameters parameters, float3 worldPosition, float3 normal) {
    if (starCosine(star, worldPosition, normal) <= 0.0) {
        return float3(0.0);
    }

    float3 toCamera = normalize(-worldPosition);
    float3 toStar = normalize(star.positionAndRadius.xyz - worldPosition);
    HapkeGeometry geometry;
    geometry.incidenceCosine = clamp(dot(toStar, normal), 1e-4, 1.0);
    geometry.emissionCosine = clamp(dot(toCamera, normal), 1e-4, 1.0);
    geometry.phaseAngle = acos(clamp(dot(toStar, toCamera), -1.0, 1.0));
    return hapkeReflectance(parameters, geometry) * starIlluminance(star, worldPosition);
}

[shader("fragment")]
FragmentOutput fragmentMain(MeshVaryings input, uint primitive: SV_PrimitiveID) {
    ditherVisibility(input);
    HapkeData material = currentMaterial<HapkeData>();
    float3 albedo = material.baseColor * input.tint;
    float3 normal = normalize(input.draw.normal);
    HapkeParameters parameters = hapkeParameters(material, albedo);
    float3 luminance = starlightLuminance(albedo);

    for (uint index = 0; index < constants.frame.starCount; index++) {
        StarData star = constants.frame.stars[index];
        float3 bounce =
            groundBounceIlluminance(star, input.draw.worldPosition, normal, input.groundUp, input.groundAlbedo);
        luminance += reflectedLuminance(star, parameters, input.draw.worldPosition, normal);
        luminance += lambertLuminance(albedo, bounce, 1.0);
    }

    MaterialSample sample;
    sample.albedo = albedo;
    sample.roughness = material.macroscopicRoughness;
    return meshOutput(luminance, sample, input, primitive);
}
