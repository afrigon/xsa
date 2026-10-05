import common.bloom;

// Must match bloomWeightSum in tonemap.slang.
static const float widerLevelWeight = 0.5;

// Adds the smaller level onto this one through a 3×3 tent filter (weights 1-2-1 per axis), at half weight per level
// so the glow concentrates near its source like a real lens's scatter, with only a faint wide skirt.
[shader("compute")]
[numthreads(8, 8, 1)]
void computeMain(uint3 pixel: SV_DispatchThreadID) {
    RWTexture2D<float4> target = storageImages[constants.targetImage];
    uint2 size;
    target.GetDimensions(size.x, size.y);
    if (any(pixel.xy >= size)) {
        return;
    }
    float2 uv = (float2(pixel.xy) + 0.5) / float2(size);
    float2 texel = 1.0 / float2(size);

    float3 upsampled = sampleSource(uv) * 4.0;
    upsampled += (sampleSource(uv + texel * float2(-1.0, 0.0)) + sampleSource(uv + texel * float2(1.0, 0.0)) +
                  sampleSource(uv + texel * float2(0.0, -1.0)) + sampleSource(uv + texel * float2(0.0, 1.0))) *
                 2.0;
    upsampled += sampleSource(uv + texel * float2(-1.0, -1.0)) + sampleSource(uv + texel * float2(1.0, -1.0)) +
                 sampleSource(uv + texel * float2(-1.0, 1.0)) + sampleSource(uv + texel * float2(1.0, 1.0));
    target[pixel.xy] = float4(target[pixel.xy].rgb + upsampled / 16.0 * widerLevelWeight, 1.0);
}
