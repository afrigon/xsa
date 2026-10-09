import common.frame;

[[vk::binding(0, 0)]]
SamplerState clampingSampler;

[[vk::binding(3, 0)]]
Texture2D textures[];

[[vk::binding(4, 0)]]
[format("rgba16f")]
RWTexture2D<float4> storageImages[];

struct TaaConstants {
    FrameData *frame;
    uint colorTexture;
    uint motionTexture;
    uint depthTexture;
    uint historyTexture;
    uint outputImage;
    uint historyValid;
    float historyExposureScale;
    float currentFrameWeight;
};

[[vk::push_constant]]
ConstantBuffer<TaaConstants> constants;

static const float3 luminanceWeights = float3(0.2126, 0.7152, 0.0722);
// History further than this many standard deviations from the neighborhood mean is pulled back to it.
static const float clipStandardDeviations = 1.0;
static const float minimumClipExtent = 1e-5;
static const int neighborhoodRadius = 1;
static const float neighborhoodSize = 9.0;

float3 toYCoCg(float3 rgb) {
    return float3(0.25 * rgb.r + 0.5 * rgb.g + 0.25 * rgb.b,
                  0.5 * rgb.r - 0.5 * rgb.b,
                  -0.25 * rgb.r + 0.5 * rgb.g - 0.25 * rgb.b);
}

float3 fromYCoCg(float3 ycocg) {
    return float3(ycocg.x + ycocg.y - ycocg.z, ycocg.x + ycocg.z, ycocg.x - ycocg.y - ycocg.z);
}

float luminance(float3 color) {
    return dot(color, luminanceWeights);
}

float3 sampleHistory(float2 uv) {
    return textures[constants.historyTexture].SampleLevel(clampingSampler, uv, 0.0).rgb;
}

// Catmull-Rom filtering keeps the history sharp where bilinear filtering would blur it a little every frame. Its 4×4
// taps fold into bilinear fetches; the four corner fetches carry so little weight that they are left out.
float3 sampleHistoryCatmullRom(float2 uv, float2 size) {
    float2 position = uv * size;
    float2 center = floor(position - 0.5) + 0.5;
    float2 f = position - center;
    float2 w0 = f * (-0.5 + f * (1.0 - 0.5 * f));
    float2 w1 = 1.0 + f * f * (-2.5 + 1.5 * f);
    float2 w2 = f * (0.5 + f * (2.0 - 1.5 * f));
    float2 w3 = f * f * (-0.5 + 0.5 * f);
    float2 w12 = w1 + w2;
    float2 uv0 = (center - 1.0) / size;
    float2 uv12 = (center + w2 / w12) / size;
    float2 uv3 = (center + 2.0) / size;

    float3 color = sampleHistory(float2(uv12.x, uv0.y)) * w12.x * w0.y;
    color += sampleHistory(float2(uv0.x, uv12.y)) * w0.x * w12.y;
    color += sampleHistory(uv12) * w12.x * w12.y;
    color += sampleHistory(float2(uv3.x, uv12.y)) * w3.x * w12.y;
    color += sampleHistory(float2(uv12.x, uv3.y)) * w12.x * w3.y;
    float weight = w12.x * w0.y + w0.x * w12.y + w12.x * w12.y + w3.x * w12.y + w12.x * w3.y;
    return color / weight;
}

// Moves the color along the line to the box's center until it lies inside, which keeps its hue unlike a clamp.
float3 clipToBox(float3 color, float3 center, float3 extent) {
    float3 offset = color - center;
    float3 units = abs(offset / max(extent, minimumClipExtent));
    float largest = max(units.x, max(units.y, units.z));
    return largest > 1.0 ? center + offset / largest : color;
}

[shader("compute")]
[numthreads(8, 8, 1)]
void computeMain(uint3 pixel: SV_DispatchThreadID) {
    FrameData frame = *constants.frame;
    float2 size = frame.viewportSize;
    if (any(float2(pixel.xy) >= size)) {
        return;
    }

    int2 center = int2(pixel.xy);
    int2 lastPixel = int2(size) - 1;
    float3 current = textures[constants.colorTexture].Load(int3(center, 0)).rgb;
    float3 sum = float3(0.0);
    float3 sumOfSquares = float3(0.0);
    // Reverse-Z: the nearest surface has the greatest depth. Its motion keeps edges moving with the foreground.
    float nearestDepth = -1.0;
    int2 nearestPixel = center;
    for (int y = -neighborhoodRadius; y <= neighborhoodRadius; y++) {
        for (int x = -neighborhoodRadius; x <= neighborhoodRadius; x++) {
            int2 neighbor = clamp(center + int2(x, y), int2(0), lastPixel);
            float3 color = toYCoCg(textures[constants.colorTexture].Load(int3(neighbor, 0)).rgb);
            sum += color;
            sumOfSquares += color * color;
            float depth = textures[constants.depthTexture].Load(int3(neighbor, 0)).r;
            if (depth > nearestDepth) {
                nearestDepth = depth;
                nearestPixel = neighbor;
            }
        }
    }

    RWTexture2D<float4> output = storageImages[constants.outputImage];
    float2 motion = textures[constants.motionTexture].Load(int3(nearestPixel, 0)).rg;
    float2 historyUv = (float2(pixel.xy) + 0.5) / size - motion;
    if (constants.historyValid == 0 || any(historyUv < 0.0) || any(historyUv > 1.0)) {
        output[pixel.xy] = float4(current, 1.0);
        return;
    }

    float3 mean = sum / neighborhoodSize;
    float3 deviation = sqrt(max(sumOfSquares / neighborhoodSize - mean * mean, 0.0));
    float3 history = max(sampleHistoryCatmullRom(historyUv, size) * constants.historyExposureScale, 0.0);
    history = fromYCoCg(clipToBox(toYCoCg(history), mean, deviation * clipStandardDeviations));

    // Weighting by inverse luminance blends as if tonemapped, so one very bright sample cannot dominate the pixel.
    float currentWeight = constants.currentFrameWeight / (1.0 + luminance(current));
    float historyWeight = (1.0 - constants.currentFrameWeight) / (1.0 + luminance(history));
    float3 resolved = (current * currentWeight + history * historyWeight) / (currentWeight + historyWeight);
    output[pixel.xy] = float4(resolved, 1.0);
}
