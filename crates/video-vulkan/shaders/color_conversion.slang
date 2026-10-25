// Converts a rendered frame to the encoder's 4:2:0 input: BT.709 Y'CbCr in limited range, as the stream's VUI
// signals. One thread covers a 2×2 block: four luma samples and the chroma sample they share.

// Must match ConversionConstants in src/conversion_constants.rs.
struct PushConstants {
    int2 sourceExtent;
    uint encodeTransfer;
};

[[vk::binding(0, 0)]]
Texture2D<float4> source;

[[vk::binding(1, 0)]]
[[vk::image_format("r8")]]
RWTexture2D<float> luma;

[[vk::binding(2, 0)]]
[[vk::image_format("rg8")]]
RWTexture2D<float2> chroma;

[[vk::push_constant]]
ConstantBuffer<PushConstants> pushConstants;

static const float3 lumaWeights = float3(0.2126, 0.7152, 0.0722);
// BT.709: Cb and Cr divide the colour difference by twice one minus the blue and red weights.
static const float blueDifferenceScale = 1.8556;
static const float redDifferenceScale = 1.5748;
// Limited range: luma spans 16 to 235 and chroma 16 to 240 of 255, centred on 128.
static const float lumaOffset = 16.0;
static const float lumaRange = 219.0;
static const float chromaOffset = 128.0;
static const float chromaRange = 224.0;

// The sRGB transfer function, for sources whose loads decode to linear light.
float3 encodeTransfer(float3 linear) {
    return select(linear <= 0.0031308, linear * 12.92, 1.055 * pow(linear, 1.0 / 2.4) - 0.055);
}

[shader("compute")]
[numthreads(8, 8, 1)]
void computeMain(uint3 id: SV_DispatchThreadID) {
    uint2 lumaSize;
    luma.GetDimensions(lumaSize.x, lumaSize.y);
    int2 base = int2(id.xy) * 2;

    if (any(base >= int2(lumaSize))) {
        return;
    }

    float3 sum = 0.0;

    for (int y = 0; y < 2; y++) {
        for (int x = 0; x < 2; x++) {
            int2 position = base + int2(x, y);
            // Past the frame, the edge repeats so the cropped padding costs almost nothing to encode.
            float3 color = source.Load(int3(min(position, pushConstants.sourceExtent - 1), 0)).rgb;

            if (pushConstants.encodeTransfer != 0) {
                color = encodeTransfer(saturate(color));
            }

            luma[position] = (lumaOffset + lumaRange * dot(color, lumaWeights)) / 255.0;
            sum += color;
        }
    }

    float3 average = sum * 0.25;
    float averageLuma = dot(average, lumaWeights);
    float2 difference =
        float2((average.b - averageLuma) / blueDifferenceScale, (average.r - averageLuma) / redDifferenceScale);
    chroma[id.xy] = (chromaOffset + chromaRange * difference) / 255.0;
}
