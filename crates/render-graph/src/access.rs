use ash::vk;

const WRITE_ACCESS: vk::AccessFlags2 = vk::AccessFlags2::from_raw(
    vk::AccessFlags2::SHADER_STORAGE_WRITE.as_raw()
        | vk::AccessFlags2::COLOR_ATTACHMENT_WRITE.as_raw()
        | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE.as_raw()
        | vk::AccessFlags2::TRANSFER_WRITE.as_raw(),
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Access {
    pub stages: vk::PipelineStageFlags2,
    pub access: vk::AccessFlags2,
    pub layout: vk::ImageLayout,
}

impl Access {
    // Bindless descriptors bake the layout an image is sampled in, so it must not depend on the step.
    pub fn sampled_layout(usage: vk::ImageUsageFlags) -> vk::ImageLayout {
        if usage.contains(vk::ImageUsageFlags::STORAGE) {
            vk::ImageLayout::GENERAL
        } else {
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        }
    }

    pub fn write_access(&self) -> vk::AccessFlags2 {
        self.access & WRITE_ACCESS
    }
}
