mod auto_exposure;
mod command_recorder;
mod frame;
mod frame_context;
mod gpu_context;
mod gpu_data;
mod material;
mod passes;
mod render_pass;
mod render_targets;
mod scene;
mod shader_binaries;
mod texture;
mod tonemapper;

pub use auto_exposure::AutoExposure;
pub use material::{HapkeParameters, Material, Shader, ShadingModel};
pub use scene::{MaterialHandle, ObjectHandle, Scene, SceneObject};
pub use shader_binaries::ShaderBinaries;
pub use texture::{ColorSpace, CubeMapHandle, TextureHandle};
pub use tonemapper::Tonemapper;

use std::path::Path;

use ash::vk;
use glam::{Mat4, Vec2, Vec3};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::camera::Camera;
use crate::config::{Config, DebugConfig, RenderConfig};
use crate::vulkan::{Image, SAMPLED_LAYOUT, Swapchain};
use auto_exposure::HISTOGRAM_BINS;
use command_recorder::CommandRecorder;
use frame::Frame;
use frame_context::FrameContext;
use gpu_context::GpuContext;
use gpu_data::{FrameData, ObjectData};
use material::MaterialData;
use passes::{BloomPass, ForwardPass, HistogramPass, TonemapPass};
use render_pass::RenderPass;
use render_targets::RenderTargets;
use texture::DdsImage;

const FRAMES_IN_FLIGHT: usize = 2;
const INITIAL_EXPOSURE_EV100: f32 = 15.0;
// Background light from stars and zodiacal light, in lux.
const STARLIGHT_ILLUMINANCE: f32 = 2e-4;
const INITIAL_OBJECT_CAPACITY: usize = 1024;
const MATERIAL_CAPACITY: usize = 256;

pub struct Renderer {
    scene: Scene,
    render: RenderConfig,
    debug: DebugConfig,
    exposure: AutoExposure,
    histogram: Vec<u32>,
    object_data: Vec<ObjectData>,
    material_data: Vec<MaterialData>,
    object_capacity: usize,
    bloom_texture: u32,
    cube_maps: Vec<Image>,
    textures: Vec<Image>,
    passes: Vec<Box<dyn RenderPass>>,
    targets: RenderTargets,
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
        let targets = RenderTargets::new(&mut gpu, swapchain.extent())?;
        let bloom = BloomPass::new(&mut gpu, shaders, swapchain.extent())?;
        let bloom_texture = bloom.texture();
        let passes: Vec<Box<dyn RenderPass>> = vec![
            Box::new(ForwardPass::new(&mut gpu, shaders)?),
            Box::new(HistogramPass::new(&gpu, shaders)?),
            Box::new(bloom),
            Box::new(TonemapPass::new(&gpu, shaders, swapchain.format())?),
        ];

        Ok(Self {
            scene: Scene::default(),
            render: config.render.clone(),
            debug: config.debug.clone(),
            exposure: AutoExposure::new(config.render.exposure.clone(), INITIAL_EXPOSURE_EV100),
            histogram: vec![0; HISTOGRAM_BINS],
            object_data: Vec::with_capacity(INITIAL_OBJECT_CAPACITY),
            material_data: Vec::with_capacity(MATERIAL_CAPACITY),
            object_capacity: INITIAL_OBJECT_CAPACITY,
            bloom_texture,
            cube_maps: Vec::new(),
            textures: Vec::new(),
            passes,
            targets,
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

    pub fn configure(&mut self, render: &RenderConfig, debug: &DebugConfig) {
        self.render = render.clone();
        self.debug = debug.clone();
        self.exposure.configure(&render.exposure);

        if debug.wireframe && !self.supports_wireframe() {
            tracing::warn!("wireframe is unsupported by this device");
            self.debug.wireframe = false;
        }
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

        self.update_exposure()?;
        self.write_frame_data(camera)?;
        self.record_frame(image_index)?;
        self.submit_and_present(image_index)?;
        self.frames[self.frame_index].histogram_ready = true;
        self.frame_index = (self.frame_index + 1) % FRAMES_IN_FLIGHT;

        Ok(())
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
        let aspect_ratio = extent.width as f32 / extent.height as f32;
        let clip_from_view = camera.clip_from_view(aspect_ratio);
        let view_projection = clip_from_view * camera.view_rotation();
        let frame_data = FrameData {
            view_projection,
            world_from_clip: view_projection.inverse(),
            sun_position: (self.scene.sun_position - camera.position).as_vec3().extend(1.0),
            sun_intensity: self.scene.sun_intensity.extend(0.0),
            viewport_size: Vec2::new(extent.width as f32, extent.height as f32),
            exposure: self.exposure.exposure(),
            hdr_texture: self.targets.hdr_texture,
            tonemapper: self.render.tonemapper.shader_id(),
            starlight_illuminance: STARLIGHT_ILLUMINANCE,
            bloom_texture: self.bloom_texture,
            bloom_strength: self.render.bloom.effective_strength(),
            shading_model: self.debug.shading_model.shader_id(),
            padding: [0; 3],
        };

        self.object_data.clear();

        for object in self.scene.objects() {
            let camera_relative = object.position - camera.position;
            self.object_data.push(ObjectData {
                world_from_model: Mat4::from_scale_rotation_translation(
                    Vec3::splat(object.scale as f32),
                    object.orientation.as_quat(),
                    camera_relative.as_vec3(),
                ),
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

    fn record_frame(&mut self, image_index: u32) -> anyhow::Result<()> {
        let device = self.gpu.device.handle();
        let frame = &self.frames[self.frame_index];

        unsafe {
            device.reset_command_pool(frame.command_pool, vk::CommandPoolResetFlags::empty())?;
            let begin_info = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            device.begin_command_buffer(frame.command_buffer, &begin_info)?;
        }

        let context = FrameContext {
            recorder: CommandRecorder::new(&self.gpu.device, frame.command_buffer),
            frame,
            targets: &self.targets,
            render: &self.render,
            debug: &self.debug,
            scene: &self.scene,
            descriptor_set: self.gpu.bindless.set(),
            output_image: self.swapchain.image(image_index),
            output_view: self.swapchain.image_view(image_index),
        };

        for pass in &mut self.passes {
            pass.record(&context)?;
        }

        unsafe { device.end_command_buffer(frame.command_buffer) }?;

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

        self.targets.recreate(&mut self.gpu, self.swapchain.extent())?;

        for pass in &mut self.passes {
            pass.resize(&mut self.gpu, &self.targets)?;
        }

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
            for pass in &mut self.passes {
                pass.destroy(gpu);
            }

            for image in self.cube_maps.iter_mut().chain(&mut self.textures) {
                image.destroy(&gpu.device, &mut gpu.allocator);
            }

            self.targets.destroy(gpu);

            for frame in &mut self.frames {
                frame.destroy(&gpu.device, &mut gpu.allocator);
            }

            self.swapchain.destroy(&gpu.device);
        }
    }
}
