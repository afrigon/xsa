import common.frame;

[[vk::binding(3, 0)]]
Texture2D textures[];

// Must match the histogram constants in client/src/renderer/exposure.rs.
static const uint binCount = 256;
static const float minimumLog2Luminance = -16.0;
static const float maximumLog2Luminance = 32.0;

struct HistogramConstants {
    FrameData *frame;
    uint *histogram;
};

[[vk::push_constant]]
ConstantBuffer<HistogramConstants> constants;

groupshared uint localBins[binCount];

// Bin 0 holds pixels that receive no light at all; bins 1 to 255 split the log2 luminance range evenly.
uint binIndex(float luminance) {
    if (luminance <= 0.0) {
        return 0;
    }
    float position = saturate((log2(luminance) - minimumLog2Luminance) / (maximumLog2Luminance - minimumLog2Luminance));
    return 1 + uint(position * float(binCount - 2));
}

[shader("compute")]
[numthreads(16, 16, 1)]
void computeMain(uint3 pixel: SV_DispatchThreadID, uint threadIndex: SV_GroupIndex) {
    localBins[threadIndex] = 0;
    GroupMemoryBarrierWithGroupSync();

    FrameData frame = *constants.frame;
    if (all(pixel.xy < uint2(frame.viewportSize))) {
        float3 color = textures[frame.sceneColorTexture].Load(int3(int2(pixel.xy), 0)).rgb;
        float luminance = dot(color, float3(0.2126, 0.7152, 0.0722)) / frame.exposure;
        InterlockedAdd(localBins[binIndex(luminance)], 1);
    }
    GroupMemoryBarrierWithGroupSync();

    uint count = localBins[threadIndex];
    if (count != 0) {
        InterlockedAdd(constants.histogram[threadIndex], count);
    }
}
