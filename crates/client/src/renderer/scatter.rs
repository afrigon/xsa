mod model_check;
mod scatter_buffers;
mod scatter_frame;
mod scatter_generator;
mod scatter_list;
mod scatter_mesh_draw;
mod scatter_page;
mod scatter_request;
mod scatter_result;
mod scatter_stream;
mod scatter_streams;
mod scatter_view;

pub(in crate::renderer) use scatter_buffers::ScatterBuffers;
pub(in crate::renderer) use scatter_frame::ScatterFrame;
pub(in crate::renderer) use scatter_streams::ScatterStreams;
pub(in crate::renderer) use scatter_view::ScatterView;

// Must match `noList` in scatter/cull.slang.
pub(in crate::renderer) const NO_LIST: u32 = u32::MAX;
// Must match `MeshInstanceData` in mesh/instance.slang, which only the GPU writes.
pub(in crate::renderer) const MESH_INSTANCE_BYTES: u64 = 144;
// A `VkDrawIndexedIndirectCommand`.
pub(in crate::renderer) const DRAW_ARGUMENTS_BYTES: u64 = 20;
