import common.bloom;

static const float3 luminanceWeights = float3(0.2126, 0.7152, 0.0722);

// Jimenez's 13-sample downsample: five overlapping 2×2 groups, the center group weighted 0.5 and the corners 0.125.
// On the first level each group is also weighted by 1 / (1 + luminance) (the Karis average), so a single extremely
// bright pixel cannot flicker as it moves across the 2×2 grid.
[shader("compute")]
[numthreads(8, 8, 1)]
void computeMain(uint3 pixel : SV_DispatchThreadID)
{
    RWTexture2D<float4> target = storageImages[constants.targetImage];
    uint2 size;
    target.GetDimensions(size.x, size.y);
    if (any(pixel.xy >= size))
    {
        return;
    }
    float2 uv = (float2(pixel.xy) + 0.5) / float2(size);
    float2 texel = 0.5 / float2(size);

    float3 a = sampleSource(uv + texel * float2(-2.0, 2.0));
    float3 b = sampleSource(uv + texel * float2(0.0, 2.0));
    float3 c = sampleSource(uv + texel * float2(2.0, 2.0));
    float3 d = sampleSource(uv + texel * float2(-2.0, 0.0));
    float3 e = sampleSource(uv);
    float3 f = sampleSource(uv + texel * float2(2.0, 0.0));
    float3 g = sampleSource(uv + texel * float2(-2.0, -2.0));
    float3 h = sampleSource(uv + texel * float2(0.0, -2.0));
    float3 i = sampleSource(uv + texel * float2(2.0, -2.0));
    float3 j = sampleSource(uv + texel * float2(-1.0, 1.0));
    float3 k = sampleSource(uv + texel * float2(1.0, 1.0));
    float3 l = sampleSource(uv + texel * float2(-1.0, -1.0));
    float3 m = sampleSource(uv + texel * float2(1.0, -1.0));

    float3 groups[5] = {
        (a + b + d + e) * 0.25,
        (b + c + e + f) * 0.25,
        (d + e + g + h) * 0.25,
        (e + f + h + i) * 0.25,
        (j + k + l + m) * 0.25,
    };
    float groupWeights[5] = { 0.125, 0.125, 0.125, 0.125, 0.5 };

    float3 sum = float3(0.0);
    float totalWeight = 0.0;
    for (int group = 0; group < 5; group++)
    {
        float weight = groupWeights[group];
        if (constants.karisAverage != 0)
        {
            weight /= 1.0 + dot(groups[group], luminanceWeights);
        }
        sum += groups[group] * weight;
        totalWeight += weight;
    }
    target[pixel.xy] = float4(sum / totalWeight, 1.0);
}
