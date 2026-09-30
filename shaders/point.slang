import common.scene;

static const float2 corners[6] = {
    float2(-1.0, -1.0),
    float2(1.0, -1.0),
    float2(1.0, 1.0),
    float2(-1.0, -1.0),
    float2(1.0, 1.0),
    float2(-1.0, 1.0),
};

struct PointOutput
{
    float4 position : SV_Position;
};

[shader("vertex")]
PointOutput vertexMain(uint vertexIndex : SV_VertexID)
{
    FrameData frame = *constants.frame;
    float4 center = mul(frame.viewProjection, mul(currentObject().worldFromModel, float4(0.0, 0.0, 0.0, 1.0)));
    float2 offset = corners[vertexIndex] * frame.pointDiameter / frame.viewportSize;
    PointOutput output;
    output.position = center + float4(offset * center.w, 0.0, 0.0);
    return output;
}

[shader("fragment")]
float4 fragmentMain(PointOutput input) : SV_Target
{
    return float4(currentObject().color.rgb, 1.0);
}
