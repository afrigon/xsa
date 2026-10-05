import common.scene;

[shader("vertex")]
VertexOutput vertexMain(VertexInput input)
{
    return transformVertex(input);
}

[shader("fragment")]
SurfaceOutput fragmentMain(VertexOutput input, uint primitive : SV_PrimitiveID)
{
    uint hash = primitive * 2654435761u;
    float3 color = float3(hash & 0xff, (hash >> 8) & 0xff, (hash >> 16) & 0xff) / 255.0;
    return surfaceOutput(float4(color, 1.0), input);
}
