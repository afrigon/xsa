import common.scene;

[[vk::binding(0, 0)]]
SamplerState clampingSampler;

[[vk::binding(3, 0)]]
Texture2D textures[];

// Level 0 of the bloom image holds every level weighted 1, 1/2, 1/4…: the sum over BLOOM_LEVELS = 6 levels in
// client/src/renderer/bloom.rs with widerLevelWeight = 0.5 in bloom-upsample.slang.
static const float bloomWeightSum = 1.96875;

// Must match Tonemapper::shader_id in client/src/renderer/tonemapper.rs.
static const uint tonemapperAgx = 0;
static const uint tonemapperAgxPunchy = 1;
static const uint tonemapperKhronosNeutral = 2;

struct FullscreenOutput
{
    float4 position : SV_Position;
};

[shader("vertex")]
FullscreenOutput vertexMain(uint vertexIndex : SV_VertexID)
{
    float2 clip = float2((vertexIndex << 1) & 2, vertexIndex & 2) * 2.0 - 1.0;
    FullscreenOutput output;
    output.position = float4(clip, 0.0, 1.0);
    return output;
}

// AgX in Benjamin Wrensch's polynomial fit of Blender's default contrast curve; matrices are given by rows.
float3 agx(float3 color, bool punchy)
{
    float3x3 inset = float3x3(
        float3(0.842479062253094, 0.0423282422610123, 0.0423756549057051),
        float3(0.0784335999999992, 0.878468636469772, 0.0784336),
        float3(0.0792237451477643, 0.0791661274605434, 0.879142973793104));
    float3x3 outset = float3x3(
        float3(1.19687900512017, -0.0528968517574562, -0.0529716355144438),
        float3(-0.0980208811401368, 1.15190312990417, -0.0980434501171241),
        float3(-0.0990297440797205, -0.0989611768448433, 1.15107367264116));
    float minimumEv = -12.47393;
    float maximumEv = 4.026069;

    float3 x = mul(color, inset);
    x = clamp(log2(max(x, 1e-10)), minimumEv, maximumEv);
    x = (x - minimumEv) / (maximumEv - minimumEv);
    float3 x2 = x * x;
    float3 x4 = x2 * x2;
    x = 15.5 * x4 * x2 - 40.14 * x4 * x + 31.96 * x4 - 6.868 * x2 * x + 0.4298 * x2 + 0.1191 * x - 0.00232;
    if (punchy)
    {
        // Blender's Punchy look: more contrast and saturation around the pixel's luminance.
        float luminance = dot(x, float3(0.2126, 0.7152, 0.0722));
        x = pow(max(x, 0.0), 1.35);
        x = luminance + 1.4 * (x - luminance);
    }
    x = mul(x, outset);
    // AgX's output is display-encoded; decode it so the sRGB swapchain's encoding restores it.
    return pow(saturate(x), 2.2);
}

// Khronos PBR Neutral, from the Khronos reference implementation.
float3 khronosNeutral(float3 color)
{
    float startCompression = 0.8 - 0.04;
    float desaturation = 0.15;
    float lowest = min(color.r, min(color.g, color.b));
    float offset = lowest < 0.08 ? lowest - 6.25 * lowest * lowest : 0.04;
    color -= offset;
    float peak = max(color.r, max(color.g, color.b));
    if (peak < startCompression)
    {
        return color;
    }
    float d = 1.0 - startCompression;
    float newPeak = 1.0 - d * d / (peak + d - startCompression);
    color *= newPeak / peak;
    float g = 1.0 - 1.0 / (desaturation * (peak - newPeak) + 1.0);
    return lerp(color, float3(newPeak), g);
}

[shader("fragment")]
float4 fragmentMain(FullscreenOutput input) : SV_Target
{
    FrameData frame = *constants.frame;
    float3 color = textures[frame.sceneColorTexture].Load(int3(int2(input.position.xy), 0)).rgb;
    if (frame.bloomStrength > 0.0)
    {
        float2 uv = input.position.xy / frame.viewportSize;
        float3 bloom = textures[frame.bloomTexture].SampleLevel(clampingSampler, uv, 0.0).rgb / bloomWeightSum;
        color = lerp(color, bloom, frame.bloomStrength);
    }
    switch (frame.tonemapper)
    {
    case tonemapperAgx:
        color = agx(color, false);
        break;
    case tonemapperAgxPunchy:
        color = agx(color, true);
        break;
    case tonemapperKhronosNeutral:
        color = khronosNeutral(color);
        break;
    default:
        color = saturate(color);
        break;
    }
    return float4(color, 1.0);
}
