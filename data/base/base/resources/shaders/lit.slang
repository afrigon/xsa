import common.scene;

struct LitData {
    float3 baseColor;
    uint padding;
};

[shader("vertex")]
VertexOutput vertexMain(VertexInput input) {
    return transformVertex(input);
}

[shader("fragment")]
SurfaceOutput fragmentMain(VertexOutput input) {
    float3 albedo = currentMaterial<LitData>().baseColor;
    float3 normal = normalize(input.normal);
    float3 luminance = starlightLuminance(albedo);
    float illuminance = 0.0;

    for (uint index = 0; index < constants.frame.starCount; index++) {
        StarData star = constants.frame.stars[index];
        float cosine = starCosine(star, input.worldPosition, normal);
        luminance += lambertLuminance(albedo, starIlluminance(star, input.worldPosition), cosine);
        illuminance += receivedIlluminance(star, input.worldPosition, normal);
    }

    float exposure = constants.frame.exposure;
    return surfaceOutput(float4(luminance * exposure, 1.0), illuminance * exposure, input);
}
