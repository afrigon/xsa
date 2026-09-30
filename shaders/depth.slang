import common.scene;

[shader("vertex")]
VertexOutput vertexMain(VertexInput input)
{
    return transformVertex(input);
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input) : SV_Target
{
    static const float maximumDistanceExponent = 12.0;
    float distance = max(length(input.worldPosition), 1.0);
    float brightness = 1.0 - saturate(log10(distance) / maximumDistanceExponent);
    return float4(brightness.xxx, 1.0);
}
