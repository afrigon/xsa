mod barriers;
mod exposure;
mod frame;
mod gpu_data;
mod material;
mod pipelines;
mod scene;
mod texture;

use std::path::Path;

use ash::vk;
use glam::{Mat4, Vec2, Vec3};
use winit::dpi::PhysicalSize;
use winit::raw_window_handle::HasDisplayHandle;
use winit::window::Window;

pub use material::{Material, Shader};
pub use pipelines::{HISTOGRAM_SHADER_PATH, POINT_SHADER_PATH, ShaderBinaries, TONEMAP_SHADER_PATH};
pub use scene::{MaterialHandle, ObjectHandle, Scene, SceneObject};
pub use texture::{ColorSpace, CubeMapHandle, TextureHandle};

use exposure::{AutoExposure, HISTOGRAM_BINS};
use frame::Frame;
use gpu_data::{FrameData, HistogramPushConstants, ObjectData, PushConstants};
use material::MaterialData;
use pipelines::Pipelines;

use crate::camera::Camera;
use crate::mesh;
use crate::vulkan::{
    Allocator, BindlessTextures, Buffer, Device, GraphicsPipeline, Image, ImageDescription, Instance, MemoryLocation,
    Surface, Swapchain,
};

const FRAMES_IN_FLIGHT: usize = 2;
const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;
const HDR_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;
const INITIAL_EXPOSURE_EV100: f32 = 15.0;
const HISTOGRAM_TILE_SIZE: u32 = 16;
// Background light from stars and zodiacal light, in lux.
const STARLIGHT_ILLUMINANCE: f32 = 2e-4;
const FULLSCREEN_VERTEX_COUNT: u32 = 3;
const SPHERE_SUBDIVISIONS: u32 = 64;
const INITIAL_OBJECT_CAPACITY: usize = 1024;
const MATERIAL_CAPACITY: usize = 256;
const POINT_MAXIMUM_DIAMETER_PIXELS: f32 = 2.0;
const POINT_MINIMUM_DIAMETER_PIXELS: f32 = 1.0;
const POINT_MINIMUM_INTENSITY: f32 = 0.1;
const POINT_FADE_DECADES: f64 = 3.0;
const POINT_QUAD_PIXELS: f32 = 4.0;
const POINT_VERTEX_COUNT: u32 = 6;

pub struct Renderer {
    scene: Scene,
    shader_override: Option<Shader>,
    wireframe: bool,
    tonemapping: bool,
    auto_exposure: AutoExposure,
    histogram: Vec<u32>,
    pipelines: Pipelines,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
    object_data: Vec<ObjectData>,
    material_data: Vec<MaterialData>,
    drawn_as_point: Vec<bool>,
    object_capacity: usize,
    depth: Image,
    hdr: Image,
    hdr_texture: u32,
    cube_maps: Vec<Image>,
    textures: Vec<Image>,
    bindless: BindlessTextures,
    frames: Vec<Frame>,
    frame_index: usize,
    swapchain: Swapchain,
    swapchain_outdated: bool,
    window_extent: vk::Extent2D,
    // Declaration order is drop order: allocator before device, device before surface, surface before instance.
    allocator: Allocator,
    device: Device,
    surface: Surface,
    instance: Instance,
}

struct DrawContext<'a> {
    device: &'a ash::Device,
    command_buffer: vk::CommandBuffer,
    frame: vk::DeviceAddress,
    objects: vk::DeviceAddress,
    materials: vk::DeviceAddress,
}

impl DrawContext<'_> {
    fn push(&self, pipeline: &GraphicsPipeline, object_index: usize, material_index: usize) {
        let push_constants = PushConstants {
            frame: self.frame,
            objects: self.objects,
            materials: self.materials,
            object_index: object_index as u32,
            material_index: material_index as u32,
        };
        unsafe {
            self.device.cmd_push_constants(
                self.command_buffer,
                pipeline.layout(),
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                gpu_data::as_bytes(&push_constants),
            );
        }
    }

    fn bind(&self, pipeline: &GraphicsPipeline) {
        unsafe {
            self.device
                .cmd_bind_pipeline(self.command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline.handle())
        };
    }
}

impl Renderer {
    pub fn new(window: &Window, shaders: &ShaderBinaries) -> anyhow::Result<Self> {
        let instance = Instance::new(window.display_handle()?.as_raw())?;
        let surface = Surface::new(&instance, window)?;
        let device = Device::new(&instance, &surface)?;
        let mut allocator = Allocator::new(&instance, &device)?;
        let window_extent = extent(window.inner_size());
        let swapchain = Swapchain::new(&instance, &surface, &device, window_extent, vk::SwapchainKHR::null())?;
        let depth = create_depth_image(&device, &mut allocator, swapchain.extent())?;
        let frames = (0..FRAMES_IN_FLIGHT)
            .map(|_| Frame::new(&device, &mut allocator, INITIAL_OBJECT_CAPACITY))
            .collect::<anyhow::Result<_>>()?;

        let sphere = mesh::cube_sphere(1.0, SPHERE_SUBDIVISIONS);
        let vertex_buffer = upload(
            &device,
            &mut allocator,
            "sphere vertices",
            vk::BufferUsageFlags::VERTEX_BUFFER,
            &sphere.vertices,
        )?;
        let index_buffer = upload(
            &device,
            &mut allocator,
            "sphere indices",
            vk::BufferUsageFlags::INDEX_BUFFER,
            &sphere.indices,
        )?;
        let mut bindless = BindlessTextures::new(&device)?;
        let hdr = create_hdr_image(&device, &mut allocator, swapchain.extent())?;
        let hdr_texture = bindless.add_texture(&device, hdr.view())?;
        let pipelines = Pipelines::new(&device, &bindless, shaders, HDR_FORMAT, swapchain.format())?;
        Ok(Self {
            scene: Scene::default(),
            shader_override: None,
            wireframe: false,
            tonemapping: true,
            auto_exposure: AutoExposure::new(INITIAL_EXPOSURE_EV100),
            histogram: vec![0; HISTOGRAM_BINS],
            pipelines,
            vertex_buffer,
            index_buffer,
            index_count: sphere.indices.len() as u32,
            object_data: Vec::with_capacity(INITIAL_OBJECT_CAPACITY),
            material_data: Vec::with_capacity(MATERIAL_CAPACITY),
            drawn_as_point: Vec::with_capacity(INITIAL_OBJECT_CAPACITY),
            object_capacity: INITIAL_OBJECT_CAPACITY,
            depth,
            hdr,
            hdr_texture,
            cube_maps: Vec::new(),
            textures: Vec::new(),
            bindless,
            frames,
            frame_index: 0,
            swapchain,
            swapchain_outdated: false,
            window_extent,
            allocator,
            device,
            surface,
            instance,
        })
    }

    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    pub fn set_shader_override(&mut self, shader: Option<Shader>) {
        self.shader_override = shader;
    }

    pub fn toggle_wireframe(&mut self) {
        if self.device.extended_dynamic_state3().is_some() {
            self.wireframe = !self.wireframe;
        }
    }

    pub fn toggle_tonemapping(&mut self) -> bool {
        self.tonemapping = !self.tonemapping;
        self.tonemapping
    }

    pub fn adjust_exposure(&mut self, stops: f32) {
        self.auto_exposure.adjust(stops);
    }

    pub fn toggle_instant_exposure(&mut self) -> bool {
        self.auto_exposure.toggle_instant()
    }

    pub fn toggle_auto_exposure(&mut self) -> bool {
        self.auto_exposure.toggle()
    }

    pub fn auto_exposure_enabled(&self) -> bool {
        self.auto_exposure.enabled()
    }

    pub fn exposure_ev100(&self) -> f32 {
        self.auto_exposure.ev100()
    }

    pub fn exposure_compensation(&self) -> f32 {
        self.auto_exposure.compensation()
    }

    pub fn load_cube_map(&mut self, name: &str, path: &Path) -> anyhow::Result<CubeMapHandle> {
        let mut image = texture::upload_cube_map(&self.device, &mut self.allocator, name, path)?;
        match self.bindless.add_cube(&self.device, image.view()) {
            Ok(index) => {
                self.cube_maps.push(image);
                Ok(CubeMapHandle::new(index))
            }
            Err(err) => {
                unsafe { image.destroy(&self.device, &mut self.allocator) };
                Err(err)
            }
        }
    }

    pub fn load_texture(&mut self, name: &str, path: &Path, color_space: ColorSpace) -> anyhow::Result<TextureHandle> {
        let mut image = texture::upload_texture(&self.device, &mut self.allocator, name, path, color_space)?;
        match self.bindless.add_texture(&self.device, image.view()) {
            Ok(index) => {
                self.textures.push(image);
                Ok(TextureHandle::new(index))
            }
            Err(err) => {
                unsafe { image.destroy(&self.device, &mut self.allocator) };
                Err(err)
            }
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.window_extent = extent(size);
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
        self.record_frame(&self.frames[self.frame_index], image_index)?;
        self.submit_and_present(image_index)?;
        self.frames[self.frame_index].histogram_ready = true;
        self.frame_index = (self.frame_index + 1) % FRAMES_IN_FLIGHT;
        Ok(())
    }

    fn update_exposure(&mut self) -> anyhow::Result<()> {
        let frame = &self.frames[self.frame_index];
        if frame.histogram_ready {
            frame.histogram.read(&mut self.histogram)?;
            self.auto_exposure.update(&self.histogram);
        }
        Ok(())
    }

    fn acquire_image(&mut self) -> anyhow::Result<Option<u32>> {
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
            point_quad_size: POINT_QUAD_PIXELS,
            exposure: exposure(self.auto_exposure.ev100()),
            hdr_texture: self.hdr_texture,
            tonemapping: u32::from(self.tonemapping),
            starlight_illuminance: STARLIGHT_ILLUMINANCE,
            padding: 0,
        };

        let pixels_per_unit_angle = f64::from(clip_from_view.y_axis.y.abs() * extent.height as f32 / 2.0);
        self.drawn_as_point.clear();
        self.object_data.clear();
        for object in self.scene.objects() {
            let camera_relative = object.position - camera.position;
            let diameter_pixels = 2.0 * object.scale / camera_relative.length() * pixels_per_unit_angle;
            let point_size = point_size(diameter_pixels);
            self.drawn_as_point
                .push(diameter_pixels < f64::from(POINT_MAXIMUM_DIAMETER_PIXELS));
            self.object_data.push(ObjectData {
                world_from_model: Mat4::from_scale_rotation_translation(
                    Vec3::splat(object.scale as f32),
                    object.orientation.as_quat(),
                    camera_relative.as_vec3(),
                ),
                point_diameter: POINT_MINIMUM_DIAMETER_PIXELS
                    + (POINT_MAXIMUM_DIAMETER_PIXELS - POINT_MINIMUM_DIAMETER_PIXELS) * point_size,
                point_intensity: POINT_MINIMUM_INTENSITY + (1.0 - POINT_MINIMUM_INTENSITY) * point_size,
                padding: [0.0; 2],
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

    fn record_frame(&self, frame: &Frame, image_index: u32) -> anyhow::Result<()> {
        let device = self.device.handle();
        let command_buffer = frame.command_buffer;
        let image = self.swapchain.image(image_index);
        unsafe {
            device.reset_command_pool(frame.command_pool, vk::CommandPoolResetFlags::empty())?;
            let begin_info = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            device.begin_command_buffer(command_buffer, &begin_info)?;
        }
        unsafe { device.cmd_fill_buffer(command_buffer, frame.histogram.handle(), 0, vk::WHOLE_SIZE, 0) };
        barriers::cleared_to_compute(device, command_buffer, frame.histogram.handle());
        barriers::sampled_to_color_attachment(device, command_buffer, self.hdr.handle());
        barriers::to_depth_attachment(device, command_buffer, self.depth.handle());
        self.begin_rendering(command_buffer, self.hdr.view(), Some(self.depth.view()));

        let context = DrawContext {
            device,
            command_buffer,
            frame: frame.frame_data.device_address(),
            objects: frame.objects.device_address(),
            materials: frame.materials.device_address(),
        };
        self.record_objects(&context);
        self.record_skybox(&context);
        self.record_points(&context);
        unsafe { device.cmd_end_rendering(command_buffer) };

        barriers::color_attachment_to_sampled(device, command_buffer, self.hdr.handle());
        self.record_histogram(command_buffer, frame);
        barriers::to_color_attachment(device, command_buffer, image);
        self.begin_rendering(command_buffer, self.swapchain.image_view(image_index), None);
        self.record_tonemap(&context);
        unsafe { device.cmd_end_rendering(command_buffer) };
        barriers::to_present(device, command_buffer, image);
        unsafe { device.end_command_buffer(command_buffer) }?;
        Ok(())
    }

    fn begin_rendering(&self, command_buffer: vk::CommandBuffer, color: vk::ImageView, depth: Option<vk::ImageView>) {
        let device = self.device.handle();
        let extent = self.swapchain.extent();
        let color_attachments = [vk::RenderingAttachmentInfo::default()
            .image_view(color)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue { float32: CLEAR_COLOR },
            })];
        let depth_attachment = depth.map(|view| {
            vk::RenderingAttachmentInfo::default()
                .image_view(view)
                .image_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .clear_value(vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue { depth: 0.0, stencil: 0 },
                })
        });
        let mut rendering_info = vk::RenderingInfo::default()
            .render_area(extent.into())
            .layer_count(1)
            .color_attachments(&color_attachments);
        if let Some(depth_attachment) = &depth_attachment {
            rendering_info = rendering_info.depth_attachment(depth_attachment);
        }
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        unsafe {
            device.cmd_begin_rendering(command_buffer, &rendering_info);
            device.cmd_set_viewport(command_buffer, 0, &[viewport]);
            device.cmd_set_scissor(command_buffer, 0, &[extent.into()]);
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipelines.point().layout(),
                0,
                &[self.bindless.set()],
                &[],
            );
        }
    }

    fn record_objects(&self, context: &DrawContext) {
        self.set_polygon_mode(context, self.wireframe);
        unsafe {
            context
                .device
                .cmd_bind_vertex_buffers(context.command_buffer, 0, &[self.vertex_buffer.handle()], &[0]);
            context.device.cmd_bind_index_buffer(
                context.command_buffer,
                self.index_buffer.handle(),
                0,
                vk::IndexType::UINT32,
            );
        }
        for shader in Shader::ALL.iter().copied().filter(|shader| *shader != Shader::Skybox) {
            let pipeline = self.pipelines.shader(shader);
            let mut bound = false;
            for (object_index, object) in self.scene.objects().iter().enumerate() {
                let object_shader = self
                    .shader_override
                    .unwrap_or(self.scene.material(object.material).shader());
                if self.drawn_as_point[object_index] || object_shader != shader {
                    continue;
                }
                if !bound {
                    context.bind(pipeline);
                    bound = true;
                }
                context.push(pipeline, object_index, object.material.index());
                unsafe {
                    context
                        .device
                        .cmd_draw_indexed(context.command_buffer, self.index_count, 1, 0, 0, 0)
                };
            }
        }
        self.set_polygon_mode(context, false);
    }

    fn record_skybox(&self, context: &DrawContext) {
        let Some(skybox) = self.scene.skybox else {
            return;
        };
        let pipeline = self.pipelines.shader(Shader::Skybox);
        context.bind(pipeline);
        context.push(pipeline, 0, skybox.index());
        unsafe {
            context
                .device
                .cmd_draw(context.command_buffer, FULLSCREEN_VERTEX_COUNT, 1, 0, 0)
        };
    }

    fn record_histogram(&self, command_buffer: vk::CommandBuffer, frame: &Frame) {
        let device = self.device.handle();
        let pipeline = self.pipelines.histogram();
        let extent = self.swapchain.extent();
        let push_constants = HistogramPushConstants {
            frame: frame.frame_data.device_address(),
            histogram: frame.histogram.device_address(),
        };
        unsafe {
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::COMPUTE, pipeline.handle());
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.layout(),
                0,
                &[self.bindless.set()],
                &[],
            );
            device.cmd_push_constants(
                command_buffer,
                pipeline.layout(),
                vk::ShaderStageFlags::COMPUTE,
                0,
                gpu_data::as_bytes(&push_constants),
            );
            device.cmd_dispatch(
                command_buffer,
                extent.width.div_ceil(HISTOGRAM_TILE_SIZE),
                extent.height.div_ceil(HISTOGRAM_TILE_SIZE),
                1,
            );
        }
        barriers::compute_to_host(device, command_buffer, frame.histogram.handle());
    }

    fn record_tonemap(&self, context: &DrawContext) {
        let pipeline = self.pipelines.tonemap();
        context.bind(pipeline);
        context.push(pipeline, 0, 0);
        unsafe {
            context
                .device
                .cmd_draw(context.command_buffer, FULLSCREEN_VERTEX_COUNT, 1, 0, 0)
        };
    }

    fn record_points(&self, context: &DrawContext) {
        let pipeline = self.pipelines.point();
        let mut bound = false;
        for (object_index, object) in self.scene.objects().iter().enumerate() {
            if !self.drawn_as_point[object_index] {
                continue;
            }
            if !bound {
                context.bind(pipeline);
                bound = true;
            }
            context.push(pipeline, object_index, object.material.index());
            unsafe {
                context
                    .device
                    .cmd_draw(context.command_buffer, POINT_VERTEX_COUNT, 1, 0, 0)
            };
        }
    }

    fn set_polygon_mode(&self, context: &DrawContext, wireframe: bool) {
        let Some(extended_dynamic_state3) = self.device.extended_dynamic_state3() else {
            return;
        };
        let polygon_mode = if wireframe {
            vk::PolygonMode::LINE
        } else {
            vk::PolygonMode::FILL
        };
        unsafe { extended_dynamic_state3.cmd_set_polygon_mode(context.command_buffer, polygon_mode) };
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
        unsafe {
            self.device
                .handle()
                .queue_submit2(self.device.queue(), &[submit], frame.in_flight)
        }?;

        let present_wait = [render_finished];
        let swapchains = [self.swapchain.handle()];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&present_wait)
            .swapchains(&swapchains)
            .image_indices(&image_indices);
        match unsafe {
            self.swapchain
                .loader()
                .queue_present(self.device.queue(), &present_info)
        } {
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
        unsafe { self.device.handle().device_wait_idle() }?;
        self.object_capacity = object_count.next_power_of_two();
        for frame in &mut self.frames {
            let objects = frame::create_object_buffer(&self.device, &mut self.allocator, self.object_capacity)?;
            let mut old_objects = std::mem::replace(&mut frame.objects, objects);
            unsafe { old_objects.destroy(&self.device, &mut self.allocator) };
        }
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

        let depth = create_depth_image(&self.device, &mut self.allocator, self.swapchain.extent())?;
        let mut old_depth = std::mem::replace(&mut self.depth, depth);
        unsafe { old_depth.destroy(&self.device, &mut self.allocator) };

        let hdr = create_hdr_image(&self.device, &mut self.allocator, self.swapchain.extent())?;
        self.bindless.set_texture(&self.device, self.hdr_texture, hdr.view());
        let mut old_hdr = std::mem::replace(&mut self.hdr, hdr);
        unsafe { old_hdr.destroy(&self.device, &mut self.allocator) };

        self.swapchain_outdated = false;
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.handle().device_wait_idle();
            self.pipelines.destroy(&self.device);
            for image in self.cube_maps.iter_mut().chain(&mut self.textures) {
                image.destroy(&self.device, &mut self.allocator);
            }
            self.bindless.destroy(&self.device);
            self.vertex_buffer.destroy(&self.device, &mut self.allocator);
            self.index_buffer.destroy(&self.device, &mut self.allocator);
            self.depth.destroy(&self.device, &mut self.allocator);
            self.hdr.destroy(&self.device, &mut self.allocator);
            for frame in &mut self.frames {
                frame.destroy(&self.device, &mut self.allocator);
            }
            self.swapchain.destroy(&self.device);
        }
    }
}

// Scales luminance (cd/m²) so that a scene metered at this EV100 lands mid-range, as a camera with ISO 100 would.
fn exposure(ev100: f32) -> f32 {
    1.0 / (1.2 * 2.0_f32.powf(ev100))
}

fn point_size(diameter_pixels: f64) -> f32 {
    let decades_below_sphere = (diameter_pixels / f64::from(POINT_MAXIMUM_DIAMETER_PIXELS)).log10();
    (1.0 + decades_below_sphere / POINT_FADE_DECADES).clamp(0.0, 1.0) as f32
}

fn create_depth_image(device: &Device, allocator: &mut Allocator, extent: vk::Extent2D) -> anyhow::Result<Image> {
    Image::new(
        device,
        allocator,
        &ImageDescription {
            name: "depth",
            extent,
            format: DEPTH_FORMAT,
            usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            aspect: vk::ImageAspectFlags::DEPTH,
            mip_levels: 1,
            cube: false,
        },
    )
}

fn create_hdr_image(device: &Device, allocator: &mut Allocator, extent: vk::Extent2D) -> anyhow::Result<Image> {
    Image::new(
        device,
        allocator,
        &ImageDescription {
            name: "hdr color",
            extent,
            format: HDR_FORMAT,
            usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
            aspect: vk::ImageAspectFlags::COLOR,
            mip_levels: 1,
            cube: false,
        },
    )
}

fn upload<T: Copy>(
    device: &Device,
    allocator: &mut Allocator,
    name: &str,
    usage: vk::BufferUsageFlags,
    data: &[T],
) -> anyhow::Result<Buffer> {
    let mut buffer = Buffer::new(
        device,
        allocator,
        name,
        size_of_val(data) as u64,
        usage,
        MemoryLocation::CpuToGpu,
    )?;
    buffer.write(data)?;
    Ok(buffer)
}

fn extent(size: PhysicalSize<u32>) -> vk::Extent2D {
    vk::Extent2D {
        width: size.width,
        height: size.height,
    }
}
