import common.scene;

[shader("vertex")]
VertexOutput vertexMain(VertexInput input)
{
    return transformVertex(input);
}

[shader("fragment")]
SurfaceOutput fragmentMain(VertexOutput input)
{
    return surfaceOutput(float4(currentMaterial().color.rgb * constants.frame.exposure, 1.0), input);
}
