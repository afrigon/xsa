import common.frame;

[[vk::binding(3, 0)]]
Texture2D textures[];

// Must match the histogram constants in client/src/renderer/star_visibility.rs.
static const uint binCount = 256;
static const float minimumLog2Luminance = -16.0;
static const float maximumLog2Luminance = 32.0;
// Must match the albedo histogram constants in client/src/renderer/visible_albedo.rs.
static const uint albedoBinCount = 64;
static const float minimumLog2Albedo = -8.0;
static const float maximumLog2Albedo = 2.0;

struct HistogramConstants {
    FrameData *frame;
    uint *histogram;
    uint *albedoHistogram;
    uint sceneColorTexture;
    // Output resolution, which the scene image may not share.
    uint lensTexture;
    uint illuminanceTexture;
    uint padding;
};

[[vk::push_constant]]
ConstantBuffer<HistogramConstants> constants;

groupshared uint localBins[binCount];
groupshared uint localAlbedoBins[albedoBinCount];

// Bin 0 holds pixels that receive no light at all; bins 1 to 255 split the log2 luminance range evenly.
uint binIndex(float luminance) {
    if (luminance <= 0.0) {
        return 0;
    }
    float position = saturate((log2(luminance) - minimumLog2Luminance) / (maximumLog2Luminance - minimumLog2Luminance));
    return 1 + uint(position * float(binCount - 2));
}

// The apparent albedo of a lit pixel: the luminance it shows over that of a white diffuser in the same light.
uint albedoBinIndex(float luminance, float illuminance) {
    float log2Albedo = log2(max(pi * luminance / illuminance, 1e-30));
    float position = saturate((log2Albedo - minimumLog2Albedo) / (maximumLog2Albedo - minimumLog2Albedo));
    return min(uint(position * float(albedoBinCount)), albedoBinCount - 1);
}

[shader("compute")]
[numthreads(16, 16, 1)]
void computeMain(uint3 pixel: SV_DispatchThreadID, uint threadIndex: SV_GroupIndex) {
    localBins[threadIndex] = 0;
    if (threadIndex < albedoBinCount) {
        localAlbedoBins[threadIndex] = 0;
    }
    GroupMemoryBarrierWithGroupSync();

    FrameData frame = *constants.frame;
    if (all(pixel.xy < uint2(frame.viewportSize))) {
        float3 color = textures[constants.sceneColorTexture].Load(int3(int2(pixel.xy), 0)).rgb;
        float illuminance = textures[constants.illuminanceTexture].Load(int3(int2(pixel.xy), 0)).r;
        if (illuminance > 0.0) {
            float surfaceLuminance = dot(color, float3(0.2126, 0.7152, 0.0722));
            InterlockedAdd(localAlbedoBins[albedoBinIndex(surfaceLuminance, illuminance)], 1);
        }
        if (constants.lensTexture != noTexture) {
            Texture2D lens = textures[constants.lensTexture];
            uint2 lensSize;
            lens.GetDimensions(lensSize.x, lensSize.y);
            uint2 lensPixel = (pixel.xy * lensSize) / uint2(frame.viewportSize);
            color += lens.Load(int3(int2(lensPixel), 0)).rgb;
        }
        float luminance = dot(color, float3(0.2126, 0.7152, 0.0722));
        InterlockedAdd(localBins[binIndex(luminance)], 1);
    }
    GroupMemoryBarrierWithGroupSync();

    uint count = localBins[threadIndex];
    if (count != 0) {
        InterlockedAdd(constants.histogram[threadIndex], count);
    }
    if (threadIndex < albedoBinCount && localAlbedoBins[threadIndex] != 0) {
        InterlockedAdd(constants.albedoHistogram[threadIndex], localAlbedoBins[threadIndex]);
    }
}
