use render_graph_vulkan::ImportedBufferHandle;

// The frame slot's scatter buffers the GPU writes: the counters, the indirect arguments and the instance lists the
// forward pass reads, and the counters' copy the CPU reads back.
#[derive(Clone, Copy)]
pub(in crate::renderer) struct ScatterInputs {
    pub counts: ImportedBufferHandle,
    pub arguments: ImportedBufferHandle,
    pub instances: ImportedBufferHandle,
    pub readback: ImportedBufferHandle,
}
