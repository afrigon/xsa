use ash::vk;
use winit::dpi::PhysicalSize;
use winit::raw_window_handle::HasDisplayHandle;
use winit::window::Window;

use crate::vulkan::{Device, GraphicsPipeline, Instance, Surface, Swapchain};

const FRAMES_IN_FLIGHT: usize = 2;
const CLEAR_COLOR: [f32; 4] = [0.02, 0.05, 0.12, 1.0];
const TRIANGLE_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/triangle.spv"));

pub struct Renderer {
    triangle: GraphicsPipeline,
    frames: Vec<Frame>,
    frame_index: usize,
    swapchain: Swapchain,
    swapchain_outdated: bool,
    window_extent: vk::Extent2D,
    // Declaration order is drop order: the device must go before the surface, the surface before the instance.
    device: Device,
    surface: Surface,
    instance: Instance,
}

impl Renderer {
    pub fn new(window: &Window) -> anyhow::Result<Self> {
        let instance = Instance::new(window.display_handle()?.as_raw())?;
        let surface = Surface::new(&instance, window)?;
        let device = Device::new(&instance, &surface)?;
        let window_extent = extent(window.inner_size());
        let swapchain = Swapchain::new(&instance, &surface, &device, window_extent, vk::SwapchainKHR::null())?;
        let triangle = GraphicsPipeline::new(&device, TRIANGLE_SHADER, swapchain.format())?;
        let frames = (0..FRAMES_IN_FLIGHT)
            .map(|_| Frame::new(&device))
            .collect::<anyhow::Result<_>>()?;

        Ok(Self {
            triangle,
            frames,
            frame_index: 0,
            swapchain,
            swapchain_outdated: false,
            window_extent,
            device,
            surface,
            instance,
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.window_extent = extent(size);
        self.swapchain_outdated = true;
    }

    pub fn draw(&mut self) -> anyhow::Result<()> {
        if self.window_extent.width == 0 || self.window_extent.height == 0 {
            return Ok(());
        }
        if self.swapchain_outdated {
            self.recreate_swapchain()?;
        }

        let device = self.device.handle();
        let frame = &self.frames[self.frame_index];
        unsafe { device.wait_for_fences(&[frame.in_flight], true, u64::MAX) }?;

        let acquired = unsafe {
            self.swapchain.loader().acquire_next_image(
                self.swapchain.handle(),
                u64::MAX,
                frame.image_acquired,
                vk::Fence::null(),
            )
        };
        let image_index = match acquired {
            Ok((image_index, suboptimal)) => {
                self.swapchain_outdated |= suboptimal;
                image_index
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.swapchain_outdated = true;
                return Ok(());
            }
            Err(err) => return Err(err.into()),
        };
        unsafe { device.reset_fences(&[frame.in_flight]) }?;

        record_frame(
            device,
            frame,
            self.triangle.handle(),
            self.swapchain.image(image_index),
            self.swapchain.image_view(image_index),
            self.swapchain.extent(),
        )?;

        let render_finished = self.swapchain.render_finished(image_index);
        let wait_semaphores = [vk::SemaphoreSubmitInfo::default()
            .semaphore(frame.image_acquired)
            .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)];
        let command_buffers = [vk::CommandBufferSubmitInfo::default().command_buffer(frame.command_buffer)];
        let signal_semaphores = [vk::SemaphoreSubmitInfo::default()
            .semaphore(render_finished)
            .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)];
        let submit = vk::SubmitInfo2::default()
            .wait_semaphore_infos(&wait_semaphores)
            .command_buffer_infos(&command_buffers)
            .signal_semaphore_infos(&signal_semaphores);
        unsafe { device.queue_submit2(self.device.queue(), &[submit], frame.in_flight) }?;

        let present_wait = [render_finished];
        let swapchains = [self.swapchain.handle()];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&present_wait)
            .swapchains(&swapchains)
            .image_indices(&image_indices);
        match unsafe { self.swapchain.loader().queue_present(self.device.queue(), &present_info) } {
            Ok(suboptimal) => self.swapchain_outdated |= suboptimal,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => self.swapchain_outdated = true,
            Err(err) => return Err(err.into()),
        }

        self.frame_index = (self.frame_index + 1) % FRAMES_IN_FLIGHT;
        Ok(())
    }

    fn recreate_swapchain(&mut self) -> anyhow::Result<()> {
        unsafe { self.device.handle().device_wait_idle() }?;
        let swapchain = Swapchain::new(
            &self.instance,
            &self.surface,
            &self.device,
            self.window_extent,
            self.swapchain.handle(),
        )?;
        let mut old_swapchain = std::mem::replace(&mut self.swapchain, swapchain);
        unsafe { old_swapchain.destroy(&self.device) };
        self.swapchain_outdated = false;
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.handle().device_wait_idle();
            self.triangle.destroy(&self.device);
            for frame in &mut self.frames {
                frame.destroy(&self.device);
            }
            self.swapchain.destroy(&self.device);
        }
    }
}

struct Frame {
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    image_acquired: vk::Semaphore,
    in_flight: vk::Fence,
}

impl Frame {
    fn new(device: &Device) -> anyhow::Result<Self> {
        let queue_family = device.queue_family();
        let device = device.handle();
        let pool_info = vk::CommandPoolCreateInfo::default().queue_family_index(queue_family);
        let command_pool = unsafe { device.create_command_pool(&pool_info, None) }?;
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer = unsafe { device.allocate_command_buffers(&allocate_info) }?[0];
        let image_acquired = unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None) }?;
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        let in_flight = unsafe { device.create_fence(&fence_info, None) }?;

        Ok(Self {
            command_pool,
            command_buffer,
            image_acquired,
            in_flight,
        })
    }

    unsafe fn destroy(&mut self, device: &Device) {
        let device = device.handle();
        unsafe {
            device.destroy_fence(self.in_flight, None);
            device.destroy_semaphore(self.image_acquired, None);
            device.destroy_command_pool(self.command_pool, None);
        }
    }
}

fn record_frame(
    device: &ash::Device,
    frame: &Frame,
    pipeline: vk::Pipeline,
    image: vk::Image,
    image_view: vk::ImageView,
    extent: vk::Extent2D,
) -> anyhow::Result<()> {
    let command_buffer = frame.command_buffer;
    unsafe {
        device.reset_command_pool(frame.command_pool, vk::CommandPoolResetFlags::empty())?;
        let begin_info = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        device.begin_command_buffer(command_buffer, &begin_info)?;

        transition_image(
            device,
            command_buffer,
            image,
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
            (vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT, vk::AccessFlags2::NONE),
            (
                vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            ),
        );

        let color_attachments = [vk::RenderingAttachmentInfo::default()
            .image_view(image_view)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue { float32: CLEAR_COLOR },
            })];
        let rendering_info = vk::RenderingInfo::default()
            .render_area(extent.into())
            .layer_count(1)
            .color_attachments(&color_attachments);
        device.cmd_begin_rendering(command_buffer, &rendering_info);
        device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        device.cmd_set_viewport(command_buffer, 0, &[viewport]);
        device.cmd_set_scissor(command_buffer, 0, &[extent.into()]);
        device.cmd_draw(command_buffer, 3, 1, 0, 0);
        device.cmd_end_rendering(command_buffer);

        transition_image(
            device,
            command_buffer,
            image,
            (vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL, vk::ImageLayout::PRESENT_SRC_KHR),
            (
                vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            ),
            (vk::PipelineStageFlags2::NONE, vk::AccessFlags2::NONE),
        );

        device.end_command_buffer(command_buffer)?;
    }
    Ok(())
}

fn transition_image(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    (old_layout, new_layout): (vk::ImageLayout, vk::ImageLayout),
    (source_stage, source_access): (vk::PipelineStageFlags2, vk::AccessFlags2),
    (destination_stage, destination_access): (vk::PipelineStageFlags2, vk::AccessFlags2),
) {
    let barriers = [vk::ImageMemoryBarrier2::default()
        .src_stage_mask(source_stage)
        .src_access_mask(source_access)
        .dst_stage_mask(destination_stage)
        .dst_access_mask(destination_access)
        .old_layout(old_layout)
        .new_layout(new_layout)
        .image(image)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .level_count(1)
                .layer_count(1),
        )];
    let dependency_info = vk::DependencyInfo::default().image_memory_barriers(&barriers);
    unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency_info) };
}

fn extent(size: PhysicalSize<u32>) -> vk::Extent2D {
    vk::Extent2D {
        width: size.width,
        height: size.height,
    }
}
