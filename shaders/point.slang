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
    float2 pixelOffset;
};

[shader("vertex")]
PointOutput vertexMain(uint vertexIndex : SV_VertexID)
{
    FrameData frame = *constants.frame;
    float4 center = mul(frame.viewProjection, mul(currentObject().worldFromModel, float4(0.0, 0.0, 0.0, 1.0)));
    float2 corner = corners[vertexIndex];
    float2 offset = corner * frame.pointQuadSize / frame.viewportSize;
    PointOutput output;
    output.position = center + float4(offset * center.w, 0.0, 0.0);
    output.pixelOffset = corner * frame.pointQuadSize / 2.0;
    return output;
}

[shader("fragment")]
float4 fragmentMain(PointOutput input) : SV_Target
{
    ObjectData object = currentObject();
    float coverage = saturate(object.pointDiameter / 2.0 + 0.5 - length(input.pixelOffset));
    return float4(object.color.rgb * object.pointIntensity * coverage, 1.0);
}
