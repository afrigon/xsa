import common.scene;

static const float2 corners[6] = {
    float2(-1.0, -1.0),
    float2(1.0, -1.0),
    float2(1.0, 1.0),
    float2(-1.0, -1.0),
    float2(1.0, 1.0),
    float2(-1.0, 1.0),
};

static const float pointLuminance = 0.8;

float luminance(float3 color)
{
    return dot(color, float3(0.2126, 0.7152, 0.0722));
}

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
    float3 color = currentMaterial().color.rgb;
    float3 hue = color / max(max(color.r, max(color.g, color.b)), 1e-6);
    float whiteMix = saturate((pointLuminance - luminance(hue)) / (1.0 - luminance(hue) + 1e-6));
    float3 pointColor = lerp(hue, float3(1.0), whiteMix);
    pointColor *= pointLuminance / max(luminance(pointColor), 1e-6);
    return float4(pointColor * object.pointIntensity * coverage, 1.0);
}
