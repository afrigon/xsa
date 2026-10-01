import common.scene;

[shader("vertex")]
VertexOutput vertexMain(VertexInput input)
{
    return transformVertex(input);
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input) : SV_Target
{
    return float4(currentMaterial().color.rgb * sunlight(input), 1.0);
}
