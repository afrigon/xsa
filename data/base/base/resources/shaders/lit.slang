import common.scene;

[shader("vertex")]
VertexOutput vertexMain(VertexInput input) {
    return transformVertex(input);
}

[shader("fragment")]
SurfaceOutput fragmentMain(VertexOutput input) {
    float3 albedo = currentMaterial().color.rgb;
    float3 normal = normalize(input.normal);
    float3 luminance = starlightLuminance(albedo);

    for (uint index = 0; index < constants.frame.starCount; index++) {
        StarData star = constants.frame.stars[index];
        float cosine = starCosine(star, input.worldPosition, normal);
        luminance += lambertLuminance(albedo, starIlluminance(star, input.worldPosition), cosine);
    }

    return surfaceOutput(float4(luminance * constants.frame.exposure, 1.0), input);
}
