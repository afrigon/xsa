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
mod gpu_timer;
mod graph_textures;
mod lens;
mod lens_lights;
mod light_occlusion;
mod light_projection;
mod material;
mod object_pose;
mod passes;
mod quad_sphere;
mod render_passes;
mod scene;
mod shader_binaries;
mod star_visibility;
mod static_buffer;
mod temporal_history;
mod terrain_buffer;
mod terrain_draw;
mod texture;
mod tonemapper;
mod white_balance;

pub use auto_exposure::AutoExposure;
pub use captured_image::CapturedImage;
pub use frame_statistics::FrameStatistics;
pub use lens_lights::LensLights;
pub use material::{
    EmissiveMaterial, HapkeParameters, LitMaterial, Material, PlanetMaterial, Shader, ShadingModel, SkyboxMaterial,
};
pub use resource_loader::texture::ColorSpace;
pub use scene::{
    LightSource, MaterialHandle, ObjectHandle, Scene, SceneObject, Star, Terrain, TerrainBufferHandle, TerrainHandle,
    TerrainHeight,
};
pub use shader_binaries::ShaderBinaries;
pub use texture::{CubeMapHandle, TextureHandle};
pub use tonemapper::Tonemapper;

use std::cell::Cell;
use std::path::Path;
use std::sync::Arc;

use ash::vk;
use glam::{DVec2, DVec3, Mat4, Vec2};
use memory_vulkan::{Buffer, Image, MemoryError, MemoryLocation, QueueSharing, Recording};
use render_graph_vulkan::{BufferState, ImageState, ImportedBuffer, ImportedImage, RenderGraph};
use winit::dpi::PhysicalSize;
use winit::window::Window;
use xsa_core::light::BACKGROUND_ILLUMINANCE;
use xui::DrawList;

use resource_loader::texture::TextureImage;
use xsa_core::terrain::HeightMap;
use xsa_packs::LensDefinition;

use crate::camera::Camera;
use crate::config::{AnimationConfig, AntialiasingKind, Config, RenderConfig};
use crate::vulkan::Swapchain;
use command_recorder::CommandRecorder;
use culler::Culler;
use frame::Frame;
use frame_context::FrameContext;
use frame_inputs::FrameInputs;
use gpu_context::GpuContext;
use gpu_data::{FrameData, ObjectData, ProjectedLightData, StarData, TerrainData, TileData};
use graph_textures::GraphTextures;
use lens::Lens;
use light_occlusion::LightOcclusion;
use light_projection::LightProjection;
use material::MaterialBuffer;
use object_pose::ObjectPose;
use passes::TaaPass;
use quad_sphere::{DetailRanges, HeightBounds, QuadSphere, SelectedTile};
use render_passes::RenderPasses;
use star_visibility::{HISTOGRAM_BINS, StarVisibility};
use temporal_history::TemporalHistory;
use terrain_buffer::TerrainBuffer;
use terrain_draw::TerrainDraw;
use texture::{BindlessSlot, SlottedTexture, TextureUpload};
use tonemapper::NO_TONEMAPPER;
use white_balance::{LUMINANCE_WEIGHTS, WhiteBalance};

const FRAMES_IN_FLIGHT: usize = 2;
const BYTES_PER_PIXEL: usize = 4;
const INITIAL_EXPOSURE_EV100: f32 = 15.0;
const INITIAL_OBJECT_CAPACITY: usize = 1024;
const INITIAL_TILE_CAPACITY: usize = 4096;
const INITIAL_STAR_CAPACITY: usize = 4;
const INITIAL_LIGHT_CAPACITY: usize = 4;
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
// The CPU reads a frame's readback buffers only after waiting for the frame on the graphics timeline.
const READ_BACK_AFTER_WAIT: BufferState = BufferState {
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
    exposure: AutoExposure,
    white_balance: WhiteBalance,
    illuminance: Option<f32>,
    star_visibility: StarVisibility,
    capture: Option<Buffer>,
    user_interface: DrawList,
    statistics: FrameStatistics,
    gpu_milliseconds: Option<f32>,
    lens_lights: LensLights,
    histogram: Vec<u32>,
    object_data: Vec<ObjectData>,
    star_data: Vec<StarData>,
    projected_light_data: Vec<ProjectedLightData>,
    light_occlusion: LightOcclusion,
    focal_length_pixels: f32,
    selected_tiles: Vec<SelectedTile>,
    terrain_draws: Vec<TerrainDraw>,
    tile_data: Vec<TileData>,
    object_capacity: usize,
    tile_capacity: usize,
    star_capacity: usize,
    light_capacity: usize,
    cube_maps: Vec<Image>,
    textures: Vec<Image>,
    materials: Vec<MaterialBuffer>,
    terrain_buffers: Vec<TerrainBuffer>,
    lens: Option<Lens>,
    // A lens loaded while its uploads are pending; it replaces `lens` once they complete.
    pending_lens: Option<Lens>,
    // Uploads whose slots are reserved and whose descriptors are written once they complete. A stopgap until
    // resources own their textures.
    pending_textures: Vec<SlottedTexture>,
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
        let gpu = GpuContext::new(window)?;
        let window_extent = Renderer::extent_of(window.inner_size());
        let swapchain = Swapchain::new(
            &gpu.memory,
            &gpu.surface,
            &gpu.device,
            window_extent,
            vk::SwapchainKHR::null(),
        )?;
        let frames = (0..FRAMES_IN_FLIGHT)
            .map(|_| {
                Frame::new(
                    &gpu.memory,
                    &gpu.device,
                    INITIAL_OBJECT_CAPACITY,
                    INITIAL_TILE_CAPACITY,
                    INITIAL_STAR_CAPACITY,
                    INITIAL_LIGHT_CAPACITY,
                )
            })
            .collect::<anyhow::Result<_>>()?;
        let mut graph = RenderGraph::new(&gpu.memory, swapchain.extent());
        let passes = RenderPasses::new(&gpu, &mut graph, shaders, swapchain.format(), FRAMES_IN_FLIGHT)?;

        Ok(Self {
            scene: Scene::default(),
            render: config.render.clone(),
            exposure: AutoExposure::new(INITIAL_EXPOSURE_EV100),
            white_balance: WhiteBalance::new(),
            illuminance: None,
            star_visibility: StarVisibility::new(),
            capture: None,
            user_interface: DrawList::default(),
            statistics: FrameStatistics::default(),
            gpu_milliseconds: None,
            lens_lights: LensLights::default(),
            histogram: vec![0; HISTOGRAM_BINS],
            object_data: Vec::with_capacity(INITIAL_OBJECT_CAPACITY),
            star_data: Vec::with_capacity(INITIAL_STAR_CAPACITY),
            projected_light_data: Vec::with_capacity(INITIAL_LIGHT_CAPACITY),
            light_occlusion: LightOcclusion::new(),
            focal_length_pixels: 0.0,
            selected_tiles: Vec::with_capacity(INITIAL_TILE_CAPACITY),
            terrain_draws: Vec::new(),
            tile_data: Vec::with_capacity(INITIAL_TILE_CAPACITY),
            object_capacity: INITIAL_OBJECT_CAPACITY,
            tile_capacity: INITIAL_TILE_CAPACITY,
            star_capacity: INITIAL_STAR_CAPACITY,
            light_capacity: INITIAL_LIGHT_CAPACITY,
            cube_maps: Vec::new(),
            textures: Vec::new(),
            materials: Vec::new(),
            terrain_buffers: Vec::new(),
            lens: None,
            pending_lens: None,
            pending_textures: Vec::new(),
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

    // Illuminance at the camera, in lux.
    pub fn set_illuminance(&mut self, illuminance: f32) {
        self.illuminance = Some(illuminance);
    }

    // Atlas updates no frame has recorded yet carry over, or their glyphs would stay blank.
    pub fn set_user_interface(&mut self, mut draw_list: DrawList) {
        draw_list
            .atlas_updates
            .splice(0..0, self.user_interface.atlas_updates.drain(..));
        self.user_interface = draw_list;
    }

    pub fn configure(&mut self, render: &RenderConfig, animations: &AnimationConfig) {
        self.render = render.clone();
        self.exposure.configure(animations.duration_scale);
        self.white_balance.configure(animations.duration_scale);
        self.star_visibility.configure(animations.duration_scale);

        if render.wireframe && !self.supports_wireframe() {
            tracing::warn!("wireframe is unsupported by this device");
            self.render.wireframe = false;
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

        // A frame's histogram is read back when its frame slot comes round again.
        antialiasing.max(FRAMES_IN_FLIGHT as u32)
    }

    // Wireframe needs the polygon mode to be dynamic state.
    pub fn supports_wireframe(&self) -> bool {
        self.gpu.device.extended_dynamic_state3().is_some()
    }

    pub fn load_cube_map(&mut self, name: &str, path: &Path) -> anyhow::Result<CubeMapHandle> {
        let image = TextureImage::read(path, ColorSpace::Srgb)?;
        let upload = TextureUpload::new(image).cube_map(&self.gpu, name)?;
        let index = self.gpu.bindless.reserve_cube()?;
        self.pending_textures
            .push(SlottedTexture::new(BindlessSlot::Cube(index), upload.into()));

        Ok(CubeMapHandle::new(index))
    }

    pub fn load_texture(&mut self, name: &str, path: &Path, color_space: ColorSpace) -> anyhow::Result<TextureHandle> {
        let image = TextureImage::read(path, color_space)?;
        self.upload_texture(name, image)
    }

    fn upload_texture(&mut self, name: &str, image: TextureImage) -> anyhow::Result<TextureHandle> {
        let texture = Renderer::slotted_texture(&mut self.gpu, name, image)?;
        let handle = TextureHandle::new(texture.slot().index());
        self.pending_textures.push(texture);

        Ok(handle)
    }

    fn slotted_texture(gpu: &mut GpuContext, name: &str, image: TextureImage) -> anyhow::Result<SlottedTexture> {
        let upload = TextureUpload::new(image).texture(gpu, name)?;
        let index = gpu.bindless.reserve_texture()?;

        Ok(SlottedTexture::new(BindlessSlot::Texture(index), upload.into()))
    }

    pub fn add_material<M: Material>(&mut self, name: &str, material: M) -> anyhow::Result<MaterialHandle> {
        self.materials.push(MaterialBuffer::new(&self.gpu, name, &material)?);
        Ok(MaterialHandle {
            index: self.materials.len() - 1,
        })
    }

    // `image` is the height map's file, uploaded with its own mip chain; `map` is its decoded top level.
    pub fn add_terrain_height(
        &mut self,
        name: &str,
        image: TextureImage,
        map: Arc<HeightMap>,
        radius: f64,
    ) -> anyhow::Result<TerrainHeight> {
        let bounds = HeightBounds::new(&map);
        let texture = self.upload_texture(&format!("{name} height map"), image)?;
        let data = TerrainData {
            height_texture: texture.index(),
            minimum_height: (map.minimum() / radius) as f32,
            height_range: ((map.maximum() - map.minimum()) / radius) as f32,
            padding: 0,
        };
        self.terrain_buffers.push(TerrainBuffer::new(&self.gpu, name, data)?);

        Ok(TerrainHeight {
            buffer: TerrainBufferHandle {
                index: self.terrain_buffers.len() - 1,
            },
            bounds,
            map,
        })
    }

    pub fn load_lens(&mut self, definition: &LensDefinition) -> anyhow::Result<()> {
        let mut lens = Lens::new(definition, |name, image| {
            Renderer::slotted_texture(&mut self.gpu, name, image)
        })?;
        lens.upload(&self.gpu)?;
        self.pending_lens = Some(lens);

        Ok(())
    }

    // Blocks until every upload queued so far completes, so the next frame may use all of them.
    pub fn finish_uploads(&mut self) -> anyhow::Result<()> {
        let gpu = &self.gpu;

        for texture in &mut self.pending_textures {
            texture.wait(&gpu.bindless, &gpu.device)?;
        }

        for material in &mut self.materials {
            material.wait()?;
        }

        for terrain in &mut self.terrain_buffers {
            terrain.wait()?;
        }

        if let Some(lens) = &mut self.pending_lens {
            lens.wait_uploads(&gpu.bindless, &gpu.device)?;
        }

        self.poll_uploads()
    }

    fn poll_uploads(&mut self) -> anyhow::Result<()> {
        let gpu = &self.gpu;
        let mut index = 0;

        while let Some(texture) = self.pending_textures.get_mut(index) {
            if !texture.poll(&gpu.bindless, &gpu.device)? {
                index += 1;
                continue;
            }

            let texture = self.pending_textures.swap_remove(index);
            let slot = texture.slot();
            let image = texture.into_image().expect("a polled ready texture holds its image");

            match slot {
                BindlessSlot::Texture(_) => self.textures.push(image),
                BindlessSlot::Cube(_) => self.cube_maps.push(image),
            }
        }

        if let Some(lens) = &mut self.pending_lens
            && lens.poll_uploads(&gpu.bindless, &gpu.device)?
        {
            self.lens = self.pending_lens.take();
        }

        Ok(())
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

        self.poll_uploads()?;
        self.ensure_object_capacity()?;
        self.ensure_star_capacity()?;
        self.ensure_light_capacity()?;

        let Some(image_index) = self.acquire_image()? else {
            return Ok(());
        };

        if self.render.antialiasing.kind != Some(AntialiasingKind::Taa) {
            self.temporal.reset();
        }

        self.update_exposure();
        self.update_white_balance(camera.position);
        self.update_star_visibility()?;
        self.update_gpu_time()?;
        self.write_frame_data(camera)?;
        let recording = self.record_frame(image_index)?;
        self.submit_and_present(image_index, recording)?;
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
        let buffer = Buffer::new(
            &self.gpu.memory,
            "capture",
            size,
            vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuToCpu,
            &QueueSharing::Exclusive,
        )?;
        self.capture = Some(buffer);
        let pixels = self.render_capture(camera, extent);
        self.capture = None;

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

        let submitted = self.frames[frame]
            .submitted
            .as_ref()
            .expect("a frame was submitted from the slot drawn above");
        self.gpu.graphics.wait(submitted.value)?;
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

    fn update_exposure(&mut self) {
        if let Some(illuminance) = self.illuminance {
            self.exposure.update(illuminance);
        }
    }

    // Adapts to the star whose light is strongest at the camera.
    fn update_white_balance(&mut self, camera_position: DVec3) {
        let dominant = self
            .scene
            .stars()
            .iter()
            .map(|star| self.scene.light_source(star.light))
            .map(|light| {
                let distance_squared = self
                    .scene
                    .object(light.object)
                    .position
                    .distance_squared(camera_position);
                let luminance = light.intensity.dot(LUMINANCE_WEIGHTS);
                (luminance as f64 / distance_squared, light.intensity / luminance)
            })
            .max_by(|a, b| a.0.total_cmp(&b.0));

        if let Some((_, color)) = dominant {
            self.white_balance.update(color);
        }
    }

    fn update_star_visibility(&mut self) -> anyhow::Result<()> {
        let frame = &self.frames[self.frame_index];

        if frame.submitted.is_some() {
            frame.histogram.read(&mut self.histogram)?;
            self.star_visibility.update(&self.histogram);
        }

        Ok(())
    }

    fn update_gpu_time(&mut self) -> anyhow::Result<()> {
        let frame = &self.frames[self.frame_index];

        if let Some(timer) = &frame.timer
            && frame.submitted.is_some()
        {
            self.gpu_milliseconds = timer.milliseconds(&self.gpu.device)?.or(self.gpu_milliseconds);
        }

        Ok(())
    }

    fn acquire_image(&mut self) -> anyhow::Result<Option<u32>> {
        let frame = &self.frames[self.frame_index];

        if let Some(submitted) = &frame.submitted {
            self.gpu.graphics.wait(submitted.value)?;
        }

        self.gpu.memory.collect()?;
        let acquired = unsafe {
            self.gpu.memory.swapchain_loader().acquire_next_image(
                self.swapchain.handle(),
                u64::MAX,
                frame.image_acquired.handle(),
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

        Ok(Some(image_index))
    }

    fn write_frame_data(&mut self, camera: &Camera) -> anyhow::Result<()> {
        let extent = self.swapchain.extent();
        let clip_from_view = camera.clip_from_view();
        let unjittered_view_projection = clip_from_view * camera.view_rotation();
        self.temporal
            .begin_frame(unjittered_view_projection, self.exposure.exposure(), camera.position);
        let jitter = if self.render.antialiasing.kind == Some(AntialiasingKind::Taa) {
            self.temporal.jitter(extent)
        } else {
            Vec2::ZERO
        };
        let view_projection = Mat4::from_translation(jitter.extend(0.0)) * unjittered_view_projection;
        let stars = self.frames[self.frame_index].stars.device_address();
        let projection = LightProjection::new(camera, extent);
        self.focal_length_pixels = projection.focal_length_pixels();
        let frame_data = FrameData {
            view_projection,
            world_from_clip: view_projection.inverse(),
            previous_view_projection: self.temporal.previous_view_projection(),
            stars,
            star_count: self.scene.stars().len() as u32,
            starfield_exposure: self.star_visibility.starfield_exposure(),
            viewport_size: Vec2::new(extent.width as f32, extent.height as f32),
            jitter,
            exposure: self.exposure.exposure(),
            tonemapper: self.render.tonemapper.map_or(NO_TONEMAPPER, Tonemapper::shader_id),
            starlight_illuminance: BACKGROUND_ILLUMINANCE as f32,
            shading_model: self.render.shading_model.shader_id(),
            vertex_spacing_angle: self.render.terrain_lod.vertex_spacing_pixels / self.focal_length_pixels,
            padding: [0; 3],
            white_balance: self.white_balance.rows(self.render.white_balance.enabled),
        };

        self.object_data.clear();

        for object in self.scene.objects() {
            let pose = ObjectPose {
                position: object.position,
                orientation: object.orientation,
                scale: object.scale,
            };
            let previous = self.temporal.record_object(pose);
            self.object_data.push(ObjectData {
                world_from_model: pose.world_from_model(camera.position),
                previous_world_from_model: previous.world_from_model(self.temporal.previous_camera_position()),
            });
        }

        self.star_data.clear();
        self.projected_light_data.clear();
        self.lens_lights = LensLights::default();
        let viewport = Vec2::new(extent.width as f32, extent.height as f32);

        for star in self.scene.stars() {
            let light = self.scene.light_source(star.light);
            let object = self.scene.object(light.object);
            self.star_data.push(StarData {
                position_and_radius: (object.position - camera.position)
                    .as_vec3()
                    .extend(object.bounding_radius as f32),
                intensity: light.intensity.extend(0.0),
            });
        }

        for light in self.scene.light_sources() {
            let object = self.scene.object(light.object);
            let camera_relative = object.position - camera.position;
            let occluders = self
                .scene
                .occluders()
                .iter()
                .filter(|&&occluder| occluder != light.object)
                .map(|&occluder| self.scene.object(occluder))
                .map(|occluder| (occluder.position - camera.position, occluder.scale));
            let visibility = self
                .light_occlusion
                .visibility(camera_relative, object.bounding_radius, occluders);
            if let Some(lens) = &self.lens {
                let projected = projection.project(
                    camera_relative,
                    object.bounding_radius,
                    light.intensity,
                    visibility,
                    lens.off_axis_exponent,
                );

                match projected {
                    None => self.lens_lights.behind += 1,
                    Some(projected)
                        if lens.reaches_screen(&projected, frame_data.exposure, viewport, self.focal_length_pixels) =>
                    {
                        self.lens_lights.drawn += 1;
                        self.projected_light_data.push(projected);
                    }
                    Some(_) => self.lens_lights.culled += 1,
                }
            }
        }

        if !self.render.terrain_lod.freeze {
            self.select_tiles(camera);
        }

        self.write_tile_data(camera);
        self.ensure_tile_capacity()?;

        let frame = &mut self.frames[self.frame_index];
        frame.frame_data.write(&[frame_data])?;
        frame.objects.write(&self.object_data)?;
        frame.tiles.write(&self.tile_data)?;
        frame.stars.write(&self.star_data)?;
        Ok(frame.projected_lights.write(&self.projected_light_data)?)
    }

    fn detail_ranges(&self, radius: f64) -> DetailRanges {
        DetailRanges::new(
            radius,
            f64::from(self.focal_length_pixels),
            f64::from(self.render.terrain_lod.vertex_spacing_pixels),
        )
    }

    // Tiles are grouped into one draw per terrain and grid size.
    fn select_tiles(&mut self, camera: &Camera) {
        let culler = Culler::new(camera, self.swapchain.extent());
        self.selected_tiles.clear();
        self.terrain_draws.clear();

        for (index, terrain) in self.scene.terrains().iter().enumerate() {
            let object = self.scene.object(terrain.object);
            let center = object.position - camera.position;

            if !culler.is_visible(center, object.bounding_radius) {
                continue;
            }

            let sphere = QuadSphere {
                radius: object.scale,
                height: terrain.height.as_ref(),
                center,
                orientation: object.orientation,
            };
            let first_tile = self.selected_tiles.len();
            sphere.select(&self.detail_ranges(object.scale), &culler, &mut self.selected_tiles);
            let tiles = &mut self.selected_tiles[first_tile..];
            tiles.sort_by_key(|tile| DetailRanges::grid_cells(tile.detail));
            let mut run_start = first_tile;

            for run in tiles.chunk_by(|first, second| {
                DetailRanges::grid_cells(first.detail) == DetailRanges::grid_cells(second.detail)
            }) {
                self.terrain_draws.push(TerrainDraw {
                    terrain: TerrainHandle { index },
                    grid_cells: DetailRanges::grid_cells(run[0].detail),
                    first_tile: run_start as u32,
                    tile_count: run.len() as u32,
                });
                run_start += run.len();
            }
        }
    }

    // Tile centers are placed in f64, relative to each frame's camera, so only offsets within a tile reach f32.
    fn write_tile_data(&mut self, camera: &Camera) {
        self.tile_data.clear();
        let previous_camera_position = self.temporal.previous_camera_position();

        for draw in &self.terrain_draws {
            let object_handle = self.scene.terrain(draw.terrain).object;
            let object = self.scene.object(object_handle);
            let previous = self.temporal.previous_pose(object_handle.index);
            let ranges = self.detail_ranges(object.scale);
            let grid = self
                .scene
                .terrain(draw.terrain)
                .height
                .as_ref()
                .map(TerrainHeight::grid);
            let first_tile = draw.first_tile as usize;

            for selected in &self.selected_tiles[first_tile..first_tile + draw.tile_count as usize] {
                let tile = selected.tile;
                let direction = tile.cube_center().normalize();
                let center = object.position - camera.position + object.orientation * (direction * object.scale);
                let previous_center =
                    previous.position - previous_camera_position + previous.orientation * (direction * previous.scale);
                let texel = grid.map_or(DVec2::ZERO, |grid| grid.texel_position(direction));
                let center_texel = texel.floor();
                self.tile_data.push(TileData {
                    center: center.as_vec3(),
                    detail: selected.detail,
                    previous_center: previous_center.as_vec3(),
                    grid_cells: draw.grid_cells,
                    cube_center: tile.cube_center().as_vec3(),
                    morph_start: ranges.morph_start(selected.detail) as f32,
                    horizontal_axis: tile.horizontal_axis().as_vec3(),
                    morph_end: ranges.morph_end(selected.detail) as f32,
                    vertical_axis: tile.vertical_axis().as_vec3(),
                    padding: 0,
                    center_texel: center_texel.as_ivec2(),
                    center_texel_fraction: (texel - center_texel).as_vec2(),
                });
            }
        }
    }

    fn record_frame(&mut self, image_index: u32) -> anyhow::Result<Recording> {
        let GpuContext {
            bindless,
            device,
            graphics,
            ..
        } = &mut self.gpu;
        let frame = &self.frames[self.frame_index];

        unsafe {
            device
                .handle()
                .reset_command_pool(frame.command_pool.handle(), vk::CommandPoolResetFlags::empty())?;
        }

        let recording = graphics.begin(frame.command_buffer)?;

        if let Some(timer) = &frame.timer {
            timer.begin(device, frame.command_buffer);
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
                initial: READ_BACK_AFTER_WAIT,
                final_state: HOST_READABLE,
            }),
            capture: self.capture.as_ref().map(|capture| {
                graph.import_buffer(ImportedBuffer {
                    name: "capture",
                    buffer: capture.handle(),
                    initial: READ_BACK_AFTER_WAIT,
                    final_state: HOST_READABLE,
                })
            }),
            projected_light_count: self.projected_light_data.len() as u32,
            lens_loaded: self.lens.is_some(),
        };
        self.passes.add_to(&mut graph, &self.render, inputs);
        let compiled = graph.compile()?;
        self.graph_textures
            .register(device, bindless, compiled.graph(), compiled.created_images())?;
        let context = FrameContext {
            recorder: CommandRecorder::new(device, frame.command_buffer),
            frame,
            frame_slot: self.frame_index,
            textures: &self.graph_textures,
            temporal: &self.temporal,
            render: &self.render,
            scene: &self.scene,
            materials: &self.materials,
            terrain_buffers: &self.terrain_buffers,
            lens: self.lens.as_ref(),
            focal_length_pixels: self.focal_length_pixels,
            terrain_draws: &self.terrain_draws,
            descriptor_set: bindless.set(),
            user_interface: &self.user_interface,
            triangles: Cell::new(0),
        };
        compiled.execute(device.handle(), frame.command_buffer, &context)?;

        if let Some(timer) = &frame.timer {
            timer.end(device, frame.command_buffer);
        }

        self.statistics = FrameStatistics {
            triangles: context.triangles.get(),
            tiles: self.selected_tiles.len(),
            deepest_depth: self.selected_tiles.iter().map(|selected| selected.tile.depth).max(),
            gpu_milliseconds: self.gpu_milliseconds,
        };
        self.user_interface.atlas_updates.clear();

        Ok(recording)
    }

    fn submit_and_present(&mut self, image_index: u32, recording: Recording) -> anyhow::Result<()> {
        let frame = &self.frames[self.frame_index];
        let render_finished = self.swapchain.render_finished(image_index);
        let wait_semaphores = [
            vk::SemaphoreSubmitInfo::default()
                .semaphore(frame.image_acquired.handle())
                .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT),
            self.gpu.transfer.visibility_wait(),
        ];
        let signal_semaphores = [vk::SemaphoreSubmitInfo::default()
            .semaphore(render_finished)
            .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)];
        let submitted = recording.submit(&wait_semaphores, &signal_semaphores)?;
        self.frames[self.frame_index].submitted = Some(submitted);

        let present_wait = [render_finished];
        let swapchains = [self.swapchain.handle()];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&present_wait)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        match self.gpu.graphics.present(present_info) {
            Ok(suboptimal) => self.swapchain_outdated |= suboptimal,
            Err(MemoryError::Vulkan(vk::Result::ERROR_OUT_OF_DATE_KHR)) => self.swapchain_outdated = true,
            Err(err) => return Err(err.into()),
        }

        Ok(())
    }

    fn ensure_object_capacity(&mut self) -> anyhow::Result<()> {
        let object_count = self.scene.objects().len();

        if object_count <= self.object_capacity {
            return Ok(());
        }

        self.object_capacity = object_count.next_power_of_two();

        for frame in &mut self.frames {
            frame.objects = Frame::create_object_buffer(&self.gpu.memory, self.object_capacity)?;
        }

        Ok(())
    }

    fn ensure_tile_capacity(&mut self) -> anyhow::Result<()> {
        let tile_count = self.tile_data.len();

        if tile_count <= self.tile_capacity {
            return Ok(());
        }

        self.tile_capacity = tile_count.next_power_of_two();

        for frame in &mut self.frames {
            frame.tiles = Frame::create_tile_buffer(&self.gpu.memory, self.tile_capacity)?;
        }

        Ok(())
    }

    fn ensure_star_capacity(&mut self) -> anyhow::Result<()> {
        let star_count = self.scene.stars().len();

        if star_count <= self.star_capacity {
            return Ok(());
        }

        self.star_capacity = star_count.next_power_of_two();

        for frame in &mut self.frames {
            frame.stars = Frame::create_star_buffer(&self.gpu.memory, self.star_capacity)?;
        }

        Ok(())
    }

    fn ensure_light_capacity(&mut self) -> anyhow::Result<()> {
        let light_count = self.scene.light_sources().len();

        if light_count <= self.light_capacity {
            return Ok(());
        }

        self.light_capacity = light_count.next_power_of_two();

        for frame in &mut self.frames {
            frame.projected_lights = Frame::create_projected_light_buffer(&self.gpu.memory, self.light_capacity)?;
        }

        Ok(())
    }

    fn recreate_swapchain(&mut self) -> anyhow::Result<()> {
        self.swapchain = Swapchain::new(
            &self.gpu.memory,
            &self.gpu.surface,
            &self.gpu.device,
            self.window_extent,
            self.swapchain.handle(),
        )?;
        let recreated = self.graph.resize(self.swapchain.extent())?;
        // Workaround: the recreated images' bindless slots are rewritten in place, which no frame still in flight may
        // read. Waiting goes away once freed slots are reused only after those frames (tracked in #285).
        self.wait_for_frames_in_flight()?;
        let gpu = &mut self.gpu;
        self.graph_textures
            .register(&gpu.device, &mut gpu.bindless, &self.graph, &recreated)?;
        self.temporal.reset();

        self.swapchain_outdated = false;

        Ok(())
    }

    fn wait_for_frames_in_flight(&self) -> anyhow::Result<()> {
        for submitted in self.frames.iter().filter_map(|frame| frame.submitted.as_ref()) {
            self.gpu.graphics.wait(submitted.value)?;
        }

        Ok(())
    }

    fn extent_of(size: PhysicalSize<u32>) -> vk::Extent2D {
        vk::Extent2D {
            width: size.width,
            height: size.height,
        }
    }
}
