import common.scene;

[[vk::binding(3, 0)]]
Texture2D textures[];

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
float3 agx(float3 color)
{
    float3x3 inset = float3x3(
        0.842479062253094, 0.0423282422610123, 0.0423756549057051,
        0.0784335999999992, 0.878468636469772, 0.0784336,
        0.0792237451477643, 0.0791661274605434, 0.879142973793104);
    float3x3 outset = float3x3(
        1.19687900512017, -0.0528968517574562, -0.0529716355144438,
        -0.0980208811401368, 1.15190312990417, -0.0980434501171241,
        -0.0990297440797205, -0.0989611768448433, 1.15107367264116);
    float minimumEv = -12.47393;
    float maximumEv = 4.026069;

    float3 x = mul(color, inset);
    x = clamp(log2(max(x, 1e-10)), minimumEv, maximumEv);
    x = (x - minimumEv) / (maximumEv - minimumEv);
    float3 x2 = x * x;
    float3 x4 = x2 * x2;
    x = 15.5 * x4 * x2 - 40.14 * x4 * x + 31.96 * x4 - 6.868 * x2 * x + 0.4298 * x2 + 0.1191 * x - 0.00232;
    x = mul(x, outset);
    // AgX's output is display-encoded; decode it so the sRGB swapchain's encoding restores it.
    return pow(saturate(x), 2.2);
}

[shader("fragment")]
float4 fragmentMain(FullscreenOutput input) : SV_Target
{
    FrameData frame = *constants.frame;
    float3 color = textures[frame.hdrTexture].Load(int3(int2(input.position.xy), 0)).rgb;
    color = frame.tonemapping != 0 ? agx(color) : saturate(color);
    return float4(color, 1.0);
}
