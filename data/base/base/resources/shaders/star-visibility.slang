import common.frame;
import common.lens;

[[vk::binding(3, 0)]]
Texture2D textures[];

struct StarVisibilityConstants {
    FrameData *frame;
    ProjectedStarData *stars;
    float *visibility;
    uint depthTexture;
    uint padding;
};

[[vk::push_constant]]
ConstantBuffer<StarVisibilityConstants> constants;

// Must match SAMPLE_GRID in client/src/renderer/passes/star_visibility_pass.rs.
static const uint sampleGrid = 8;
// A star smaller than a pixel still covers the pixels its light lands on.
static const float minimumFootprintPixels = 2.0;

groupshared uint diskSamples;
groupshared uint clearSamples;

[shader("compute")]
[numthreads(sampleGrid, sampleGrid, 1)]
void computeMain(uint3 group: SV_GroupID, uint3 thread: SV_GroupThreadID, uint threadIndex: SV_GroupIndex) {
    if (threadIndex == 0) {
        diskSamples = 0;
        clearSamples = 0;
    }

    GroupMemoryBarrierWithGroupSync();

    ProjectedStarData star = constants.stars[group.x];
    float2 offset = (float2(thread.xy) + 0.5) / float(sampleGrid) * 2.0 - 1.0;

    if (dot(offset, offset) <= 1.0) {
        InterlockedAdd(diskSamples, 1);
        float2 pixel = star.screenPosition + offset * max(star.pixelRadius, minimumFootprintPixels);
        float2 viewportSize = constants.frame.viewportSize;

        if (all(pixel >= 0.0) && all(pixel < viewportSize)) {
            float depth = textures[constants.depthTexture].Load(int3(int2(pixel), 0)).r;

            if (depth <= star.occlusionDepth) {
                InterlockedAdd(clearSamples, 1);
            }
        }
    }

    GroupMemoryBarrierWithGroupSync();

    if (threadIndex == 0) {
        constants.visibility[group.x] = float(clearSamples) / float(max(diskSamples, 1));
    }
}
