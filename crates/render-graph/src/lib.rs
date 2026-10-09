//! A render graph for Vulkan 1.3 with synchronization2 and dynamic rendering.
//!
//! Passes declare the images and buffers each of their steps uses and how; the graph culls passes nothing needs,
//! derives every barrier, layout transition and attachment load and store operation, allocates the images passes
//! create and keeps history images across frames. A frame whose declarations match an earlier one reuses its
//! compiled result.

mod access;
mod attachment;
mod barrier_source;
mod buffer_handle;
mod buffer_state;
mod buffer_usage;
mod compiled_attachment;
mod compiled_barrier;
mod compiled_buffer_barrier;
mod compiled_graph;
mod compiled_pass;
mod compiled_step;
mod declared_buffer;
mod declared_buffer_usage;
mod declared_image;
mod declared_image_usage;
mod declared_pass;
mod end_state;
mod frame_declaration;
mod graph_compiler;
mod graph_image_description;
mod history_id;
mod history_image;
mod image_handle;
mod image_id;
mod image_size;
mod image_state;
mod imported_buffer;
mod imported_image;
mod pass_declaration;
mod pass_error;
mod render_graph_error;
mod resource_usage;
mod stage;
mod subresource;

pub use attachment::Attachment;
pub use buffer_handle::BufferHandle;
pub use buffer_state::BufferState;
pub use buffer_usage::BufferUsage;
pub use graph_image_description::GraphImageDescription;
pub use history_id::HistoryId;
pub use history_image::HistoryImage;
pub use image_handle::ImageHandle;
pub use image_id::ImageId;
pub use image_size::ImageSize;
pub use image_state::ImageState;
pub use imported_buffer::ImportedBuffer;
pub use imported_image::ImportedImage;
pub use pass_declaration::PassDeclaration;
pub use pass_error::PassError;
pub use render_graph_error::RenderGraphError;
pub use resource_usage::ResourceUsage;
pub use stage::Stage;
pub use subresource::Subresource;
