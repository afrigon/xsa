import common.scene;

[shader("vertex")]
VertexOutput vertexMain(VertexInput input)
{
    return transformVertex(input);
}

[shader("fragment")]
SurfaceOutput fragmentMain(VertexOutput input)
{
    return surfaceOutput(float4(sunCosine(input.worldPosition, normalize(input.normal)).xxx, 1.0), input);
}
