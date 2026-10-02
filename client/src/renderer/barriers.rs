use ash::vk;

const DEPTH_TESTS: vk::PipelineStageFlags2 = vk::PipelineStageFlags2::from_raw(
    vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS.as_raw() | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS.as_raw(),
);

struct ImageTransition {
    image: vk::Image,
    aspect: vk::ImageAspectFlags,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    source_stage: vk::PipelineStageFlags2,
    source_access: vk::AccessFlags2,
    destination_stage: vk::PipelineStageFlags2,
    destination_access: vk::AccessFlags2,
}

pub(super) fn to_color_attachment(device: &ash::Device, command_buffer: vk::CommandBuffer, image: vk::Image) {
    record(
        device,
        command_buffer,
        &ImageTransition {
            image,
            aspect: vk::ImageAspectFlags::COLOR,
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            source_stage: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            source_access: vk::AccessFlags2::NONE,
            destination_stage: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            destination_access: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
        },
    );
}

pub(super) fn sampled_to_color_attachment(device: &ash::Device, command_buffer: vk::CommandBuffer, image: vk::Image) {
    record(
        device,
        command_buffer,
        &ImageTransition {
            image,
            aspect: vk::ImageAspectFlags::COLOR,
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            source_stage: vk::PipelineStageFlags2::FRAGMENT_SHADER | vk::PipelineStageFlags2::COMPUTE_SHADER,
            source_access: vk::AccessFlags2::NONE,
            destination_stage: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            destination_access: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
        },
    );
}

pub(super) fn color_attachment_to_sampled(device: &ash::Device, command_buffer: vk::CommandBuffer, image: vk::Image) {
    record(
        device,
        command_buffer,
        &ImageTransition {
            image,
            aspect: vk::ImageAspectFlags::COLOR,
            old_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            source_stage: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            source_access: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            destination_stage: vk::PipelineStageFlags2::FRAGMENT_SHADER | vk::PipelineStageFlags2::COMPUTE_SHADER,
            destination_access: vk::AccessFlags2::SHADER_SAMPLED_READ,
        },
    );
}

pub(super) fn to_depth_attachment(device: &ash::Device, command_buffer: vk::CommandBuffer, image: vk::Image) {
    record(
        device,
        command_buffer,
        &ImageTransition {
            image,
            aspect: vk::ImageAspectFlags::DEPTH,
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            source_stage: DEPTH_TESTS,
            source_access: vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
            destination_stage: DEPTH_TESTS,
            destination_access: vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ
                | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
        },
    );
}

pub(super) fn to_present(device: &ash::Device, command_buffer: vk::CommandBuffer, image: vk::Image) {
    record(
        device,
        command_buffer,
        &ImageTransition {
            image,
            aspect: vk::ImageAspectFlags::COLOR,
            old_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            new_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            source_stage: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            source_access: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            destination_stage: vk::PipelineStageFlags2::NONE,
            destination_access: vk::AccessFlags2::NONE,
        },
    );
}

pub(super) fn cleared_to_compute(device: &ash::Device, command_buffer: vk::CommandBuffer, buffer: vk::Buffer) {
    record_buffer(
        device,
        command_buffer,
        &BufferDependency {
            buffer,
            source_stage: vk::PipelineStageFlags2::CLEAR,
            source_access: vk::AccessFlags2::TRANSFER_WRITE,
            destination_stage: vk::PipelineStageFlags2::COMPUTE_SHADER,
            destination_access: vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_STORAGE_WRITE,
        },
    );
}

pub(super) fn compute_to_host(device: &ash::Device, command_buffer: vk::CommandBuffer, buffer: vk::Buffer) {
    record_buffer(
        device,
        command_buffer,
        &BufferDependency {
            buffer,
            source_stage: vk::PipelineStageFlags2::COMPUTE_SHADER,
            source_access: vk::AccessFlags2::SHADER_STORAGE_WRITE,
            destination_stage: vk::PipelineStageFlags2::HOST,
            destination_access: vk::AccessFlags2::HOST_READ,
        },
    );
}

struct BufferDependency {
    buffer: vk::Buffer,
    source_stage: vk::PipelineStageFlags2,
    source_access: vk::AccessFlags2,
    destination_stage: vk::PipelineStageFlags2,
    destination_access: vk::AccessFlags2,
}

fn record_buffer(device: &ash::Device, command_buffer: vk::CommandBuffer, dependency: &BufferDependency) {
    let barriers = [vk::BufferMemoryBarrier2::default()
        .src_stage_mask(dependency.source_stage)
        .src_access_mask(dependency.source_access)
        .dst_stage_mask(dependency.destination_stage)
        .dst_access_mask(dependency.destination_access)
        .buffer(dependency.buffer)
        .size(vk::WHOLE_SIZE)];
    let dependency_info = vk::DependencyInfo::default().buffer_memory_barriers(&barriers);
    unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency_info) };
}

fn record(device: &ash::Device, command_buffer: vk::CommandBuffer, transition: &ImageTransition) {
    let barriers = [vk::ImageMemoryBarrier2::default()
        .src_stage_mask(transition.source_stage)
        .src_access_mask(transition.source_access)
        .dst_stage_mask(transition.destination_stage)
        .dst_access_mask(transition.destination_access)
        .old_layout(transition.old_layout)
        .new_layout(transition.new_layout)
        .image(transition.image)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(transition.aspect)
                .level_count(1)
                .layer_count(1),
        )];
    let dependency_info = vk::DependencyInfo::default().image_memory_barriers(&barriers);
    unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency_info) };
}
