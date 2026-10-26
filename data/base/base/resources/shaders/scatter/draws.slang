// Must match `DRAWS_WORKGROUP_SIZE` in the renderer.
static const uint workgroupSize = 64;
// A `VkDrawIndexedIndirectCommand`: index count, instance count, first index, vertex offset and first instance.
static const uint argumentWords = 5;

// One submesh of a drawn scatter model, mirroring `ScatterDrawData` in the renderer.
struct ScatterDrawData {
    uint list;
    uint indexCount;
    uint firstIndex;
    uint padding;
};

struct ScatterDrawConstants {
    ScatterDrawData *draws;
    uint *counts;
    uint *arguments;
    uint drawCount;
    uint padding;
};

[[vk::push_constant]]
ConstantBuffer<ScatterDrawConstants> constants;

// Each submesh draws every instance its model's list holds.
[shader("compute")]
[numthreads(workgroupSize, 1, 1)]
void computeMain(uint3 id: SV_DispatchThreadID) {
    if (id.x >= constants.drawCount) {
        return;
    }

    ScatterDrawData draw = constants.draws[id.x];
    uint first = id.x * argumentWords;
    constants.arguments[first] = draw.indexCount;
    constants.arguments[first + 1] = constants.counts[draw.list];
    constants.arguments[first + 2] = draw.firstIndex;
    constants.arguments[first + 3] = 0;
    constants.arguments[first + 4] = 0;
}
