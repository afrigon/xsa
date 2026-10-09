mod auto_exposure;
mod captured_image;
mod command_recorder;
mod culler;
mod frame;
mod frame_context;
mod frame_inputs;
mod frame_statistics;
mod frustum;
mod gpu_context;
mod gpu_data;
mod graph_textures;
mod material;
mod passes;
mod render_passes;
mod scene;
mod shader_binaries;
mod temporal_history;
mod texture;
mod tonemapper;

pub use auto_exposure::AutoExposure;
pub use captured_image::CapturedImage;
pub use frame_statistics::FrameStatistics;
pub use material::{HapkeParameters, Material, Shader, ShadingModel};
pub use scene::{MaterialHandle, ObjectHandle, Scene, SceneObject};
pub use shader_binaries::ShaderBinaries;
pub use texture::{ColorSpace, CubeMapHandle, TextureHandle};
pub use tonemapper::Tonemapper;

use std::cell::Cell;
use std::path::Path;

use ash::vk;
use glam::{Mat4, Vec2, Vec3};
use render_graph::{BufferState, ImageState, ImportedBuffer, ImportedImage, RenderGraph};
use winit::dpi::PhysicalSize;
use winit::window::Window;
use xui::DrawList;

use crate::camera::Camera;
use crate::config::{AntialiasingKind, Config, DebugConfig, RenderConfig};
use crate::vulkan::{Buffer, Image, MemoryLocation, SAMPLED_LAYOUT, Swapchain};
use auto_exposure::HISTOGRAM_BINS;
use command_recorder::CommandRecorder;
use culler::Culler;
use frame::Frame;
use frame_context::FrameContext;
use frame_inputs::FrameInputs;
use gpu_context::GpuContext;
use gpu_data::{FrameData, ObjectData};
use graph_textures::GraphTextures;
use material::MaterialData;
use passes::TaaPass;
use render_passes::RenderPasses;
use temporal_history::TemporalHistory;
use texture::DdsImage;

const FRAMES_IN_FLIGHT: usize = 2;
const BYTES_PER_PIXEL: usize = 4;
const INITIAL_EXPOSURE_EV100: f32 = 15.0;
// Background light from stars and zodiacal light, in lux.
const STARLIGHT_ILLUMINANCE: f32 = 2e-4;
const INITIAL_OBJECT_CAPACITY: usize = 1024;
const MATERIAL_CAPACITY: usize = 256;
// Presentation waits on the acquire semaphore at the color attachment output stage.
const SWAPCHAIN_ACQUIRED: ImageState = ImageState {
    stages: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
    access: vk::AccessFlags2::NONE,
    layout: vk::ImageLayout::UNDEFINED,
};
const SWAPCHAIN_PRESENTABLE: ImageState = ImageState {
    stages: vk::PipelineStageFlags2::NONE,
    access: vk::AccessFlags2::NONE,
    layout: vk::ImageLayout::PRESENT_SRC_KHR,
};
// The CPU reads a frame's readback buffers only after waiting on its fence.
const READ_BACK_AFTER_FENCE: BufferState = BufferState {
    stages: vk::PipelineStageFlags2::NONE,
    access: vk::AccessFlags2::NONE,
};
const HOST_READABLE: BufferState = BufferState {
    stages: vk::PipelineStageFlags2::HOST,
    access: vk::AccessFlags2::HOST_READ,
};

pub struct Renderer {
    scene: Scene,
    render: RenderConfig,
    debug: DebugConfig,
    exposure: AutoExposure,
    capture: Option<Buffer>,
    user_interface: DrawList,
    statistics: FrameStatistics,
    histogram: Vec<u32>,
    object_data: Vec<ObjectData>,
    visible_objects: Vec<ObjectHandle>,
    material_data: Vec<MaterialData>,
    object_capacity: usize,
    cube_maps: Vec<Image>,
    textures: Vec<Image>,
    passes: RenderPasses,
    graph: RenderGraph,
    graph_textures: GraphTextures,
    temporal: TemporalHistory,
    frames: Vec<Frame>,
    frame_index: usize,
    swapchain: Swapchain,
    swapchain_outdated: bool,
    window_extent: vk::Extent2D,
    gpu: GpuContext,
}

impl Renderer {
    pub fn new(window: &Window, shaders: &ShaderBinaries, config: &Config) -> anyhow::Result<Self> {
        let mut gpu = GpuContext::new(window)?;
        let window_extent = Renderer::extent_of(window.inner_size());
        let swapchain = Swapchain::new(
            &gpu.instance,
            &gpu.surface,
            &gpu.device,
            window_extent,
            vk::SwapchainKHR::null(),
        )?;
        let frames = (0..FRAMES_IN_FLIGHT)
            .map(|_| Frame::new(&gpu.device, &mut gpu.allocator, INITIAL_OBJECT_CAPACITY))
            .collect::<anyhow::Result<_>>()?;
        let mut graph = RenderGraph::new(swapchain.extent());
        let passes = RenderPasses::new(&mut gpu, &mut graph, shaders, swapchain.format(), FRAMES_IN_FLIGHT)?;

        Ok(Self {
            scene: Scene::default(),
            render: config.render.clone(),
            debug: config.debug.clone(),
            exposure: AutoExposure::new(config.render.exposure.clone(), INITIAL_EXPOSURE_EV100),
            capture: None,
            user_interface: DrawList::default(),
            statistics: FrameStatistics::default(),
            histogram: vec![0; HISTOGRAM_BINS],
            object_data: Vec::with_capacity(INITIAL_OBJECT_CAPACITY),
            visible_objects: Vec::with_capacity(INITIAL_OBJECT_CAPACITY),
            material_data: Vec::with_capacity(MATERIAL_CAPACITY),
            object_capacity: INITIAL_OBJECT_CAPACITY,
            cube_maps: Vec::new(),
            textures: Vec::new(),
            passes,
            graph,
            graph_textures: GraphTextures::default(),
            temporal: TemporalHistory::default(),
            frames,
            frame_index: 0,
            swapchain,
            swapchain_outdated: false,
            window_extent,
            gpu,
        })
    }

    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    pub fn statistics(&self) -> FrameStatistics {
        self.statistics
    }

    // Atlas updates no frame has recorded yet carry over, or their glyphs would stay blank.
    pub fn set_user_interface(&mut self, mut draw_list: DrawList) {
        draw_list
            .atlas_updates
            .splice(0..0, self.user_interface.atlas_updates.drain(..));
        self.user_interface = draw_list;
    }

    pub fn configure(&mut self, render: &RenderConfig, debug: &DebugConfig) {
        self.render = render.clone();
        self.debug = debug.clone();
        self.exposure
            .configure(&render.exposure, debug.animation_duration_scale);

        if debug.wireframe && !self.supports_wireframe() {
            tracing::warn!("wireframe is unsupported by this device");
            self.debug.wireframe = false;
        }
    }

    // For a camera cut: the accumulated image no longer matches anything on screen.
    pub fn reset_history(&mut self) {
        self.temporal.reset();
    }

    // Frames to render after a change before the image reflects it: temporal effects carry earlier frames forward.
    pub fn settling_frames(&self) -> u32 {
        let antialiasing = match self.render.antialiasing.kind {
            Some(AntialiasingKind::Taa) => TaaPass::settling_frames(),
            _ => 0,
        };

        antialiasing.max(self.exposure.settling_frames())
    }

    // Wireframe needs the polygon mode to be dynamic state.
    pub fn supports_wireframe(&self) -> bool {
        self.gpu.device.extended_dynamic_state3().is_some()
    }

    pub fn load_cube_map(&mut self, name: &str, path: &Path) -> anyhow::Result<CubeMapHandle> {
        let gpu = &mut self.gpu;
        let mut image =
            DdsImage::read(path, ColorSpace::Srgb)?.upload_cube_map(&gpu.device, &mut gpu.allocator, name)?;

        match gpu.bindless.add_cube(&gpu.device, image.view()) {
            Ok(index) => {
                self.cube_maps.push(image);
                Ok(CubeMapHandle::new(index))
            }
            Err(err) => {
                unsafe { image.destroy(&gpu.device, &mut gpu.allocator) };
                Err(err)
            }
        }
    }

    pub fn load_texture(&mut self, name: &str, path: &Path, color_space: ColorSpace) -> anyhow::Result<TextureHandle> {
        let gpu = &mut self.gpu;
        let mut image = DdsImage::read(path, color_space)?.upload_texture(&gpu.device, &mut gpu.allocator, name)?;

        match gpu.bindless.add_texture(&gpu.device, image.view(), SAMPLED_LAYOUT) {
            Ok(index) => {
                self.textures.push(image);
                Ok(TextureHandle::new(index))
            }
            Err(err) => {
                unsafe { image.destroy(&gpu.device, &mut gpu.allocator) };
                Err(err)
            }
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.window_extent = Renderer::extent_of(size);
        self.swapchain_outdated = true;
    }

    pub fn draw(&mut self, camera: &Camera) -> anyhow::Result<()> {
        if self.window_extent.width == 0 || self.window_extent.height == 0 {
            return Ok(());
        }

        if self.swapchain_outdated {
            self.recreate_swapchain()?;
        }

        self.ensure_object_capacity()?;

        let Some(image_index) = self.acquire_image()? else {
            return Ok(());
        };

        if self.render.antialiasing.kind != Some(AntialiasingKind::Taa) {
            self.temporal.reset();
        }

        self.update_exposure()?;
        self.write_frame_data(camera)?;
        self.cull_objects(camera);
        self.record_frame(image_index)?;
        self.submit_and_present(image_index)?;
        self.frames[self.frame_index].histogram_ready = true;
        self.frame_index = (self.frame_index + 1) % FRAMES_IN_FLIGHT;

        Ok(())
    }

    // Renders one frame now and blocks until its pixels are back on the CPU.
    pub fn capture(&mut self, camera: &Camera) -> anyhow::Result<CapturedImage> {
        anyhow::ensure!(
            self.swapchain.supports_capture(),
            "this display does not allow copying the rendered image"
        );

        if self.swapchain_outdated {
            self.recreate_swapchain()?;
        }

        let extent = self.swapchain.extent();
        let size = u64::from(extent.width) * u64::from(extent.height) * BYTES_PER_PIXEL as u64;
        let gpu = &mut self.gpu;
        let buffer = Buffer::new(
            &gpu.device,
            &mut gpu.allocator,
            "capture",
            size,
            vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuToCpu,
        )?;
        self.capture = Some(buffer);
        let pixels = self.render_capture(camera, extent);
        let mut buffer = self.capture.take().expect("the capture buffer was set above");
        unsafe { buffer.destroy(&self.gpu.device, &mut self.gpu.allocator) };

        Ok(CapturedImage {
            width: extent.width,
            height: extent.height,
            rgba: pixels?,
        })
    }

    fn render_capture(&mut self, camera: &Camera, extent: vk::Extent2D) -> anyhow::Result<Vec<u8>> {
        let frame = self.frame_index;
        self.draw(camera)?;

        // No image was acquired because the swapchain went out of date: recreate it and try once more.
        if self.frame_index == frame && self.swapchain_outdated {
            self.recreate_swapchain()?;
            anyhow::ensure!(
                self.swapchain.extent() == extent,
                "the window changed size while capturing"
            );
            self.draw(camera)?;
        }

        anyhow::ensure!(
            self.frame_index != frame,
            "the window is not visible, nothing was rendered"
        );

        let device = self.gpu.device.handle();
        unsafe { device.wait_for_fences(&[self.frames[frame].in_flight], true, u64::MAX) }?;
        let mut pixels = vec![0_u8; extent.width as usize * extent.height as usize * BYTES_PER_PIXEL];
        self.capture
            .as_ref()
            .expect("the capture buffer is set while capturing")
            .read(&mut pixels)?;

        match self.swapchain.format() {
            vk::Format::B8G8R8A8_SRGB | vk::Format::B8G8R8A8_UNORM => {
                for pixel in pixels.as_chunks_mut::<BYTES_PER_PIXEL>().0 {
                    pixel.swap(0, 2);
                }
            }
            vk::Format::R8G8B8A8_SRGB | vk::Format::R8G8B8A8_UNORM => {}
            format => anyhow::bail!("capturing a {format:?} swapchain is not supported"),
        }

        Ok(pixels)
    }

    fn update_exposure(&mut self) -> anyhow::Result<()> {
        let frame = &self.frames[self.frame_index];

        if frame.histogram_ready {
            frame.histogram.read(&mut self.histogram)?;
            self.exposure.update(&self.histogram);
        }

        Ok(())
    }

    fn acquire_image(&mut self) -> anyhow::Result<Option<u32>> {
        let device = self.gpu.device.handle();
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
                return Ok(None);
            }
            Err(err) => return Err(err.into()),
        };
        unsafe { device.reset_fences(&[frame.in_flight]) }?;

        Ok(Some(image_index))
    }

    fn write_frame_data(&mut self, camera: &Camera) -> anyhow::Result<()> {
        let extent = self.swapchain.extent();
        let clip_from_view = camera.clip_from_view();
        let unjittered_view_projection = clip_from_view * camera.view_rotation();
        self.temporal
            .begin_frame(unjittered_view_projection, self.exposure.exposure());
        let jitter = if self.render.antialiasing.kind == Some(AntialiasingKind::Taa) {
            self.temporal.jitter(extent)
        } else {
            Vec2::ZERO
        };
        let view_projection = Mat4::from_translation(jitter.extend(0.0)) * unjittered_view_projection;
        let frame_data = FrameData {
            view_projection,
            world_from_clip: view_projection.inverse(),
            previous_view_projection: self.temporal.previous_view_projection(),
            sun_position: (self.scene.sun_position - camera.position).as_vec3().extend(1.0),
            sun_intensity: self.scene.sun_intensity.extend(0.0),
            viewport_size: Vec2::new(extent.width as f32, extent.height as f32),
            jitter,
            exposure: self.exposure.exposure(),
            tonemapper: self.render.tonemapper.shader_id(),
            starlight_illuminance: STARLIGHT_ILLUMINANCE,
            shading_model: self.debug.shading_model.shader_id(),
        };

        self.object_data.clear();

        for object in self.scene.objects() {
            let camera_relative = object.position - camera.position;
            let world_from_model = Mat4::from_scale_rotation_translation(
                Vec3::splat(object.scale as f32),
                object.orientation.as_quat(),
                camera_relative.as_vec3(),
            );
            self.object_data.push(ObjectData {
                world_from_model,
                previous_clip_from_model: self.temporal.record_object(world_from_model),
            });
        }

        anyhow::ensure!(
            self.scene.materials().len() <= MATERIAL_CAPACITY,
            "the scene has more than {MATERIAL_CAPACITY} materials"
        );
        self.material_data.clear();
        self.material_data
            .extend(self.scene.materials().iter().map(Material::gpu_data));

        let frame = &mut self.frames[self.frame_index];
        frame.frame_data.write(&[frame_data])?;
        frame.objects.write(&self.object_data)?;

        frame.materials.write(&self.material_data)
    }

    fn cull_objects(&mut self, camera: &Camera) {
        let culler = Culler::new(camera, self.swapchain.extent());
        self.visible_objects.clear();
        self.visible_objects.extend(
            self.scene
                .objects()
                .iter()
                .enumerate()
                .filter(|(_, object)| culler.is_visible(object.position - camera.position, object.bounding_radius))
                .map(|(index, _)| ObjectHandle { index }),
        );
    }

    fn record_frame(&mut self, image_index: u32) -> anyhow::Result<()> {
        let GpuContext {
            bindless,
            allocator,
            device,
            ..
        } = &mut self.gpu;
        let frame = &self.frames[self.frame_index];

        unsafe {
            device
                .handle()
                .reset_command_pool(frame.command_pool, vk::CommandPoolResetFlags::empty())?;
            let begin_info = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            device
                .handle()
                .begin_command_buffer(frame.command_buffer, &begin_info)?;
        }

        let mut graph = self.graph.begin_frame();
        let inputs = FrameInputs {
            output: graph.import_image(ImportedImage {
                name: "swapchain",
                image: self.swapchain.image(image_index),
                view: self.swapchain.image_view(image_index),
                extent: self.swapchain.extent(),
                aspect: vk::ImageAspectFlags::COLOR,
                initial: SWAPCHAIN_ACQUIRED,
                final_state: SWAPCHAIN_PRESENTABLE,
            }),
            histogram: graph.import_buffer(ImportedBuffer {
                name: "luminance histogram",
                buffer: frame.histogram.handle(),
                initial: READ_BACK_AFTER_FENCE,
                final_state: HOST_READABLE,
            }),
            capture: self.capture.as_ref().map(|capture| {
                graph.import_buffer(ImportedBuffer {
                    name: "capture",
                    buffer: capture.handle(),
                    initial: READ_BACK_AFTER_FENCE,
                    final_state: HOST_READABLE,
                })
            }),
        };
        self.passes.add_to(&mut graph, &self.render, inputs);
        let compiled = graph.compile(device.handle(), allocator.gpu_allocator())?;
        self.graph_textures
            .register(device, bindless, compiled.graph(), compiled.created_images())?;
        let context = FrameContext {
            recorder: CommandRecorder::new(device, frame.command_buffer),
            frame,
            frame_slot: self.frame_index,
            textures: &self.graph_textures,
            temporal: &self.temporal,
            render: &self.render,
            debug: &self.debug,
            scene: &self.scene,
            visible_objects: &self.visible_objects,
            descriptor_set: bindless.set(),
            user_interface: &self.user_interface,
            triangles: Cell::new(0),
        };
        compiled.execute(device.handle(), frame.command_buffer, &context)?;
        unsafe { device.handle().end_command_buffer(frame.command_buffer) }?;
        self.statistics = FrameStatistics {
            triangles: context.triangles.get(),
        };
        self.user_interface.atlas_updates.clear();

        Ok(())
    }

    fn submit_and_present(&mut self, image_index: u32) -> anyhow::Result<()> {
        let frame = &self.frames[self.frame_index];
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
        let device = &self.gpu.device;
        unsafe {
            device
                .handle()
                .queue_submit2(device.queue(), &[submit], frame.in_flight)
        }?;

        let present_wait = [render_finished];
        let swapchains = [self.swapchain.handle()];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&present_wait)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        match unsafe { self.swapchain.loader().queue_present(device.queue(), &present_info) } {
            Ok(suboptimal) => self.swapchain_outdated |= suboptimal,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => self.swapchain_outdated = true,
            Err(err) => return Err(err.into()),
        }

        Ok(())
    }

    fn ensure_object_capacity(&mut self) -> anyhow::Result<()> {
        let object_count = self.scene.objects().len();

        if object_count <= self.object_capacity {
            return Ok(());
        }

        self.gpu.wait_idle()?;
        self.object_capacity = object_count.next_power_of_two();

        for frame in &mut self.frames {
            let objects = Frame::create_object_buffer(&self.gpu.device, &mut self.gpu.allocator, self.object_capacity)?;
            let mut old_objects = std::mem::replace(&mut frame.objects, objects);
            unsafe { old_objects.destroy(&self.gpu.device, &mut self.gpu.allocator) };
        }

        Ok(())
    }

    fn recreate_swapchain(&mut self) -> anyhow::Result<()> {
        self.gpu.wait_idle()?;
        let swapchain = Swapchain::new(
            &self.gpu.instance,
            &self.gpu.surface,
            &self.gpu.device,
            self.window_extent,
            self.swapchain.handle(),
        )?;
        let mut old_swapchain = std::mem::replace(&mut self.swapchain, swapchain);
        unsafe { old_swapchain.destroy(&self.gpu.device) };

        let gpu = &mut self.gpu;
        let recreated = unsafe {
            self.graph.resize(
                gpu.device.handle(),
                gpu.allocator.gpu_allocator(),
                self.swapchain.extent(),
            )
        }?;
        self.graph_textures
            .register(&gpu.device, &mut gpu.bindless, &self.graph, &recreated)?;
        self.temporal.reset();

        self.swapchain_outdated = false;

        Ok(())
    }

    fn extent_of(size: PhysicalSize<u32>) -> vk::Extent2D {
        vk::Extent2D {
            width: size.width,
            height: size.height,
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let _ = self.gpu.wait_idle();
        let gpu = &mut self.gpu;

        unsafe {
            self.passes.destroy(gpu);

            if let Err(err) = self.graph.destroy(gpu.device.handle(), gpu.allocator.gpu_allocator()) {
                tracing::error!("destroying the render graph: {err}");
            }

            for image in self.cube_maps.iter_mut().chain(&mut self.textures) {
                image.destroy(&gpu.device, &mut gpu.allocator);
            }

            for frame in &mut self.frames {
                frame.destroy(&gpu.device, &mut gpu.allocator);
            }

            self.swapchain.destroy(&gpu.device);
        }
    }
}
