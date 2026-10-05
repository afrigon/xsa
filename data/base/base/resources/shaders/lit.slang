import common.scene;

[shader("vertex")]
VertexOutput vertexMain(VertexInput input)
{
    return transformVertex(input);
}

[shader("fragment")]
SurfaceOutput fragmentMain(VertexOutput input)
{
    float cosine = sunCosine(input.worldPosition, normalize(input.normal));
    float3 luminance = lambertLuminance(currentMaterial().color.rgb, sunIlluminance(input.worldPosition), cosine);
    return surfaceOutput(float4(luminance * constants.frame.exposure, 1.0), input);
}
