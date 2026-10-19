use ash::vk;

pub struct GraphicsPipelineDescription<'a> {
    pub vertex_spirv: &'a [u8],
    pub fragment_spirv: &'a [u8],
    pub color_formats: &'a [vk::Format],
    pub depth_format: Option<vk::Format>,
    pub depth_compare_op: vk::CompareOp,
    pub depth_write: bool,
    pub cull_mode: vk::CullModeFlags,
    pub additive_blend: bool,
    // Lets the polygon mode switch to lines at draw time, where the device supports it.
    pub wireframe: bool,
    pub vertex_bindings: &'a [vk::VertexInputBindingDescription],
    pub vertex_attributes: &'a [vk::VertexInputAttributeDescription],
    pub push_constant_size: u32,
    pub descriptor_set_layouts: &'a [vk::DescriptorSetLayout],
}
