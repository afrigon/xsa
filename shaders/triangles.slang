import common.scene;

[shader("vertex")]
VertexOutput vertexMain(VertexInput input)
{
    return transformVertex(input);
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input, uint primitive : SV_PrimitiveID) : SV_Target
{
    uint hash = primitive * 2654435761u;
    float3 color = float3(hash & 0xff, (hash >> 8) & 0xff, (hash >> 16) & 0xff) / 255.0;
    return float4(color, 1.0);
}
