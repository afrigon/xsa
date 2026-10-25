mod auto_exposure;
mod capture_readback;
mod captured_image;
mod command_recorder;
mod culler;
mod debug_view;
mod forward_pipeline;
mod frame;
mod frame_context;
mod frame_inputs;
mod frame_output;
mod frame_role;
mod frame_statistics;
mod frame_target;
mod frustum;
mod gpu_context;
mod gpu_data;
mod gpu_timer;
mod graph_textures;
mod lens;
mod lens_flare;
mod lens_lights;
mod light_occlusion;
mod light_projection;
mod material;
mod noise_volume;
mod object_pose;
mod passes;
mod pending_capture;
mod quad_sphere;
mod queued_capture;
mod render_passes;
mod scene;
mod shader_binaries;
mod shading_range;
mod star_visibility;
mod static_buffer;
mod surface;
mod surface_debug_view;
mod temporal_history;
mod terrain;
mod terrain_debug_view;
mod terrain_draw;
mod tile_shape;
mod tonemapper;
mod video_output;
mod video_phase;
mod video_target;
mod white_balance;

pub use auto_exposure::AutoExposure;
pub use captured_image::CapturedImage;
pub use debug_view::DebugView;
pub use frame_statistics::FrameStatistics;
pub use frame_target::FrameTarget;
pub use lens_lights::LensLights;
pub use material::{EmissiveMaterial, HapkeMaterial, LambertMaterial, Material, Shader, SkyboxMaterial};
pub use pending_capture::PendingCapture;
pub use resource_loader::texture::ColorSpace;
pub use scene::{LightSource, MaterialHandle, ObjectHandle, Scene, SceneObject, Star, Terrain, TerrainHandle};
pub use shader_binaries::ShaderBinaries;
pub use surface::{BiomeSurfaces, SurfaceClass, SurfaceTextures};
pub use surface_debug_view::SurfaceDebugView;
pub use terrain::{TerrainLayer, TerrainPageStatistics, TerrainPages};
pub use terrain_debug_view::TerrainDebugView;
pub use tonemapper::Tonemapper;

use std::cell::Cell;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use ash::vk;
use glam::{DVec3, Mat4, Vec2, Vec3};
use memory_vulkan::{Buffer, MemoryError, MemoryLocation, Recording, TransferPriority};
use render_graph_vulkan::{
    BufferState, GraphImageDescription, ImageSize, ImageState, ImportedBuffer, ImportedImage, RenderGraph, Subresource,
};
use tokio::sync::oneshot;
use winit::dpi::PhysicalSize;
use winit::window::Window;
use xsa_core::light::BACKGROUND_ILLUMINANCE;
use xui::DrawList;

use resource_vulkan::{Residency, ResourceHandle, Resources};
use xsa_packs::LensDefinition;

use crate::camera::Camera;
use crate::config::{AntialiasingKind, Config};
use crate::resource_kinds::{LensGhost, LensStarburst};
use crate::vulkan::Swapchain;
use capture_readback::{BYTES_PER_PIXEL, CaptureReadback};
use command_recorder::CommandRecorder;
use culler::Culler;
use frame::Frame;
use frame_context::FrameContext;
use frame_inputs::FrameInputs;
use frame_output::FrameOutput;
use frame_role::FrameRole;
use gpu_context::GpuContext;
use gpu_data::{FrameData, ObjectData, ProjectedLightData, StarData, SurfaceAnchorData, TileData, TilePages};
use graph_textures::GraphTextures;
use lens::Lens;
use light_occlusion::LightOcclusion;
use light_projection::LightProjection;
use material::MaterialBuffer;
use noise_volume::NoiseVolume;
use object_pose::ObjectPose;
use passes::TaaPass;
use quad_sphere::{DetailRanges, QuadSphere, SelectedTile, TileHeightBounds};
use queued_capture::QueuedCapture;
use render_passes::RenderPasses;
use shading_range::ShadingRange;
use star_visibility::{HISTOGRAM_BINS, StarVisibility};
use temporal_history::TemporalHistory;
use terrain::{PageDetail, TilePageWriter};
use terrain_draw::TerrainDraw;
use tile_shape::TileShape;
use tonemapper::NO_TONEMAPPER;
use video_output::VideoOutput;
use video_phase::VideoPhase;
use video_target::VideoTarget;
use white_balance::{LUMINANCE_WEIGHTS, WhiteBalance};

const FRAMES_IN_FLIGHT: usize = 2;
const INITIAL_EXPOSURE_EV100: f32 = 15.0;
const INITIAL_OBJECT_CAPACITY: usize = 1024;
const INITIAL_TILE_CAPACITY: usize = 4096;
const INITIAL_STAR_CAPACITY: usize = 4;
const INITIAL_SURFACE_ANCHOR_CAPACITY: usize = 4;
const INITIAL_LIGHT_CAPACITY: usize = 4;
// Far longer than any frame the compositor asks for. Some drivers never return an image while the window is hidden
// (NVIDIA on Wayland); the frame is skipped rather than the event loop stalled.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(1);
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
    config: Config,
    exposure: AutoExposure,
    white_balance: WhiteBalance,
    illuminance: Option<f32>,
    star_visibility: StarVisibility,
    captures: Vec<QueuedCapture>,
    // The buffer the frame being recorded copies its output into, on capture frames.
    capture: Option<Buffer>,
    video: Option<VideoOutput>,
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
    tile_pages: Vec<TilePages>,
    surface_anchor_data: Vec<SurfaceAnchorData>,
    // Each terrain's slot in `surface_anchor_data`, by terrain index; None for terrains without surfaces.
    surface_anchor_slots: Vec<Option<usize>>,
    noise_volume: NoiseVolume,
    page_writer: TilePageWriter,
    // Whether terrain pages follow what tiles request, or stay as they are.
    terrain_residency: Residency,
    object_capacity: usize,
    tile_capacity: usize,
    star_capacity: usize,
    surface_anchor_capacity: usize,
    light_capacity: usize,
    materials: Vec<MaterialBuffer>,
    lens: Option<Lens>,
    passes: RenderPasses,
    graph: RenderGraph,
    graph_textures: GraphTextures,
    temporal: TemporalHistory,
    frames: Vec<Frame>,
    frame_index: usize,
    frames_rendered: u64,
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
                    INITIAL_SURFACE_ANCHOR_CAPACITY,
                )
            })
            .collect::<anyhow::Result<_>>()?;
        let mut graph = RenderGraph::new(&gpu.memory, swapchain.extent());
        let passes = RenderPasses::new(&gpu, &mut graph, shaders, swapchain.format(), FRAMES_IN_FLIGHT)?;
        let noise_volume = NoiseVolume::new(&gpu)?;

        Ok(Self {
            scene: Scene::default(),
            config: config.clone(),
            exposure: AutoExposure::new(INITIAL_EXPOSURE_EV100),
            white_balance: WhiteBalance::new(),
            illuminance: None,
            star_visibility: StarVisibility::new(),
            captures: Vec::new(),
            capture: None,
            video: None,
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
            tile_pages: Vec::with_capacity(INITIAL_TILE_CAPACITY),
            surface_anchor_data: Vec::with_capacity(INITIAL_SURFACE_ANCHOR_CAPACITY),
            surface_anchor_slots: Vec::new(),
            noise_volume,
            page_writer: TilePageWriter::new(),
            terrain_residency: Residency::Live,
            object_capacity: INITIAL_OBJECT_CAPACITY,
            tile_capacity: INITIAL_TILE_CAPACITY,
            star_capacity: INITIAL_STAR_CAPACITY,
            surface_anchor_capacity: INITIAL_SURFACE_ANCHOR_CAPACITY,
            light_capacity: INITIAL_LIGHT_CAPACITY,
            materials: Vec::new(),
            lens: None,
            passes,
            graph,
            graph_textures: GraphTextures::default(),
            temporal: TemporalHistory::default(),
            frames,
            frame_index: 0,
            frames_rendered: 0,
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

    pub fn configure(&mut self, config: &Config) {
        self.config = config.clone();
        self.terrain_residency = if config.terrain.streaming.freeze {
            Residency::Frozen
        } else {
            Residency::Live
        };
        let adaptation_duration_scale = config.animation.camera_duration_scale();
        self.exposure.configure(adaptation_duration_scale);
        self.white_balance.configure(adaptation_duration_scale);
        self.star_visibility.configure(adaptation_duration_scale);

        if config.debug.wireframe && !self.supports_wireframe() {
            tracing::warn!("wireframe is unsupported by this device");
            self.config.debug.wireframe = false;
        }
    }

    // For a camera cut: the accumulated image no longer matches anything on screen.
    pub fn reset_history(&mut self) {
        self.temporal.reset();
    }

    // Frames to render after a change before the image reflects it: temporal effects carry earlier frames forward.
    fn settling_frames(&self) -> u32 {
        let antialiasing = match self.config.video.antialiasing {
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

    // Loads into the same GPU memory, transfer queue and bindless table the renderer draws from.
    pub fn create_resources(&self) -> Resources {
        Resources::new(&self.gpu.memory, &self.gpu.transfer, &self.gpu.bindless)
    }

    pub fn add_material<M: Material + 'static>(&mut self, name: &str, material: M) -> anyhow::Result<MaterialHandle> {
        self.materials.push(MaterialBuffer::new(&self.gpu, name, material)?);
        Ok(MaterialHandle {
            index: self.materials.len() - 1,
        })
    }

    // Replaces the lens; it is drawn once its textures are loaded and its ghosts uploaded.
    pub fn load_lens(
        &mut self,
        definition: LensDefinition,
        starburst: ResourceHandle<LensStarburst>,
        ghosts: Vec<ResourceHandle<LensGhost>>,
    ) {
        self.lens = Some(Lens::new(definition, starburst, ghosts));
    }

    // Derives what loaded resources add to the scene, and polls the uploads of what was derived.
    fn poll_uploads(&mut self) -> anyhow::Result<()> {
        for material in &mut self.materials {
            material.refresh(&self.gpu)?;
        }

        for pages in self
            .scene
            .terrains()
            .iter()
            .filter_map(|terrain| terrain.pages.as_ref())
        {
            pages.update(self.terrain_residency);
        }

        if let Some(lens) = &mut self.lens {
            lens.refresh(&self.gpu)?;
        }

        Ok(())
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.window_extent = Renderer::extent_of(size);
        self.swapchain_outdated = true;
    }

    // `now` is the moment the frame shows: adaptations to the light advance by the time since the previous one.
    pub fn draw(&mut self, camera: &Camera, target: FrameTarget, now: Instant) -> anyhow::Result<()> {
        if self.window_extent.width == 0 || self.window_extent.height == 0 {
            return Ok(());
        }

        if self.swapchain_outdated {
            self.recreate_swapchain()?;
        }

        self.poll_uploads()?;
        self.passes.ensure_pipelines(&self.gpu, &self.config)?;
        self.ensure_object_capacity()?;
        self.ensure_star_capacity()?;
        self.ensure_light_capacity()?;
        self.wait_for_frame_slot()?;

        let output = match target {
            FrameTarget::Window { window } => {
                let Some(image_index) = self.acquire_image()? else {
                    return Ok(());
                };

                FrameOutput::Window { window, image_index }
            }
            FrameTarget::Offscreen => FrameOutput::Offscreen,
        };

        self.render_frame(camera, output, now, FrameRole::Display)?;
        self.render_settled_captures(camera, now);

        Ok(())
    }

    // Captures a frame once temporal effects reflect the scene. It is rendered off screen, so it is taken whether the
    // window shows or not.
    pub fn request_capture(&mut self) -> PendingCapture {
        let (sender, receiver) = oneshot::channel();
        self.captures.push(QueuedCapture {
            settling_frames: self.settling_frames(),
            sender,
        });

        PendingCapture::new(receiver)
    }

    // Every frame displayed from now on is converted, encoded and written to `path` at `fps`, until `stop_video`.
    pub fn start_video(&mut self, path: PathBuf, fps: u32) -> anyhow::Result<()> {
        anyhow::ensure!(self.video.is_none(), "a video is already being written");
        self.video = Some(VideoOutput::new(
            &self.gpu,
            path,
            self.swapchain.extent(),
            self.swapchain.format(),
            fps,
            FRAMES_IN_FLIGHT,
        )?);

        Ok(())
    }

    // Stops capturing and deletes the file, for a capture that failed.
    pub fn abort_video(&mut self) {
        if let Some(video) = self.video.take() {
            video.discard();
        }
    }

    pub fn stop_video(&mut self) {
        if let Some(video) = &mut self.video {
            video.phase = VideoPhase::Closing;
        }
    }

    // Whether the next displayed frame can be captured; without a video every frame can.
    pub fn video_ready(&self) -> bool {
        self.video.as_ref().is_none_or(VideoOutput::is_ready)
    }

    // Writes the frames encoded so far. Returns the file once a stopped video is complete, or why it failed.
    pub fn poll_video(&mut self) -> Option<anyhow::Result<PathBuf>> {
        let video = self.video.as_mut()?;

        if let Err(error) = video.write_encoded() {
            if let Some(video) = self.video.take() {
                video.discard();
            }

            return Some(Err(error.into()));
        }

        if !video.is_done() {
            return None;
        }

        let video = self.video.take()?;

        Some(video.finish().map_err(Into::into))
    }

    fn captures_display_frames(&self) -> bool {
        self.video
            .as_ref()
            .is_some_and(|video| video.phase == VideoPhase::Recording)
    }

    pub fn frames_rendered(&self) -> u64 {
        self.frames_rendered
    }

    fn render_frame(
        &mut self,
        camera: &Camera,
        output: FrameOutput,
        now: Instant,
        role: FrameRole,
    ) -> anyhow::Result<()> {
        if self.config.video.antialiasing != Some(AntialiasingKind::Taa) {
            self.temporal.reset();
        }

        self.update_exposure(now);
        self.update_white_balance(camera.position, now);
        self.update_star_visibility(now)?;
        self.update_gpu_time()?;
        self.write_frame_data(camera)?;
        let captured = self.captures_display_frames() && role == FrameRole::Display;
        let recording = self.record_frame(output, captured, camera.clear_color)?;

        match output {
            FrameOutput::Window { window, image_index } => self.submit_and_present(window, image_index, recording)?,
            FrameOutput::Offscreen => self.submit_offscreen(recording)?,
        }

        if captured && let (Some(video), Some(submitted)) = (&mut self.video, &self.frames[self.frame_index].submitted)
        {
            video.encode(submitted);
        }

        self.frame_index = (self.frame_index + 1) % FRAMES_IN_FLIGHT;
        self.frames_rendered += 1;

        for capture in &mut self.captures {
            capture.settling_frames = capture.settling_frames.saturating_sub(1);
        }

        Ok(())
    }

    fn render_settled_captures(&mut self, camera: &Camera, now: Instant) {
        let (settled, settling): (Vec<_>, Vec<_>) = std::mem::take(&mut self.captures)
            .into_iter()
            .partition(|capture| capture.settling_frames == 0);
        self.captures = settling;

        for capture in settled {
            let readback = self.render_capture(camera, now);
            let _ = capture.sender.send(readback);
        }
    }

    fn render_capture(&mut self, camera: &Camera, now: Instant) -> anyhow::Result<CaptureReadback> {
        let extent = self.swapchain.extent();
        let size = u64::from(extent.width) * u64::from(extent.height) * BYTES_PER_PIXEL as u64;
        let buffer = Buffer::new(
            &self.gpu.memory,
            "capture",
            size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::GpuOnly,
            &self.gpu.transfer.sharing(),
        )?;
        self.wait_for_frame_slot()?;
        let frame = self.frame_index;
        self.capture = Some(buffer);
        let rendered = self.render_frame(camera, FrameOutput::Offscreen, now, FrameRole::Snapshot);
        let buffer = self.capture.take().expect("the capture buffer is set while capturing");
        rendered?;
        let submitted = self.frames[frame]
            .submitted
            .clone()
            .expect("a frame was submitted from the slot rendered above");
        let readback = self
            .gpu
            .transfer
            .read_buffer(buffer, Some(submitted), TransferPriority::High);

        Ok(CaptureReadback::new(extent, self.swapchain.format(), readback))
    }

    fn update_exposure(&mut self, now: Instant) {
        if let Some(illuminance) = self.illuminance {
            self.exposure.update(illuminance, now);
        }
    }

    // Adapts to the star whose light is strongest at the camera.
    fn update_white_balance(&mut self, camera_position: DVec3, now: Instant) {
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
            self.white_balance.update(color, now);
        }
    }

    fn update_star_visibility(&mut self, now: Instant) -> anyhow::Result<()> {
        let frame = &self.frames[self.frame_index];

        if frame.submitted.is_some() {
            frame.histogram.read(&mut self.histogram)?;
            self.star_visibility.update(&self.histogram, now);
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

    // Once the slot's previous frame is done, its buffers may be rewritten and its read-backs hold that frame's results.
    fn wait_for_frame_slot(&mut self) -> anyhow::Result<()> {
        if let Some(submitted) = &self.frames[self.frame_index].submitted {
            self.gpu.graphics.wait(submitted.value)?;
        }

        self.gpu.memory.collect()?;

        Ok(())
    }

    fn acquire_image(&mut self) -> anyhow::Result<Option<u32>> {
        let acquired = unsafe {
            self.gpu.memory.swapchain_loader().acquire_next_image(
                self.swapchain.handle(),
                ACQUIRE_TIMEOUT.as_nanos() as u64,
                self.frames[self.frame_index].image_acquired.handle(),
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
            Err(vk::Result::TIMEOUT) => {
                tracing::debug!("no swapchain image became available within {ACQUIRE_TIMEOUT:?}, skipping the frame");
                return Ok(None);
            }
            Err(err) => return Err(err.into()),
        };

        Ok(Some(image_index))
    }

    fn write_frame_data(&mut self, camera: &Camera) -> anyhow::Result<()> {
        self.noise_volume.poll(&self.gpu)?;
        let extent = self.swapchain.extent();
        let clip_from_view = camera.clip_from_view();
        let unjittered_view_projection = clip_from_view * camera.view_rotation();
        self.temporal
            .begin_frame(unjittered_view_projection, self.exposure.exposure(), camera.position);
        let jitter = if self.config.video.antialiasing == Some(AntialiasingKind::Taa) {
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
            tonemapper: self
                .config
                .video
                .tonemapper
                .map_or(NO_TONEMAPPER, Tonemapper::shader_id),
            starlight_illuminance: BACKGROUND_ILLUMINANCE as f32,
            vertex_spacing_angle: self.config.terrain.lod.vertex_spacing_pixels / self.focal_length_pixels,
            pixel_angle: self.focal_length_pixels.recip(),
            terrain_height_scale: self.config.terrain.height.drawn_scale(),
            padding: [0; 2],
            white_balance: self.white_balance.rows(self.config.video.white_balance.enabled),
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
            if let Some(lens) = self.lens.as_ref().and_then(Lens::flare) {
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

        if !self.config.terrain.lod.freeze {
            self.select_tiles(camera);
        }

        self.write_tiles(camera);
        self.ensure_tile_capacity()?;
        self.write_surface_anchors(camera);
        self.ensure_surface_anchor_capacity()?;

        let frame = &mut self.frames[self.frame_index];
        frame.frame_data.write(&[frame_data])?;
        frame.objects.write(&self.object_data)?;
        frame.tiles.write(&self.tile_data)?;
        frame.tile_pages.write(&self.tile_pages)?;
        frame.stars.write(&self.star_data)?;
        frame.surface_anchors.write(&self.surface_anchor_data)?;
        Ok(frame.projected_lights.write(&self.projected_light_data)?)
    }

    fn detail_ranges(&self, radius: f64) -> DetailRanges {
        DetailRanges::new(
            radius,
            f64::from(self.focal_length_pixels),
            f64::from(self.config.terrain.lod.vertex_spacing_pixels),
        )
    }

    // Tiles are grouped into one draw per terrain, shading range and grid size. A tile is shaded with surfaces where
    // some pixel of it can magnify the colour layer past its finest texels.
    fn select_tiles(&mut self, camera: &Camera) {
        let culler = Culler::new(camera, self.swapchain.extent());
        self.selected_tiles.clear();
        self.terrain_draws.clear();

        for (index, terrain) in self.scene.terrains().iter().enumerate() {
            let object = self.scene.object(terrain.object);
            let center = object.position - camera.position;

            let heights = terrain.pages.as_ref().and_then(|pages| pages.heights());
            let height_scale = f64::from(self.config.terrain.height.drawn_scale());
            let shape = if heights.is_some() {
                TileShape::Displaced
            } else {
                TileShape::Smooth
            };
            let highest = heights
                .as_ref()
                .and_then(TileHeightBounds::overall_range)
                .map_or(0.0, |range| range.scaled(height_scale).maximum);
            let relief = terrain.surfaces.as_deref().map_or(0.0, BiomeSurfaces::relief);
            let bounding_radius = object.bounding_radius + highest + relief;

            if !culler.is_visible(center, bounding_radius) {
                continue;
            }

            let sphere = QuadSphere {
                radius: object.scale,
                height: heights.as_ref().map(|heights| heights as &dyn TileHeightBounds),
                relief,
                height_scale,
                center,
                orientation: object.orientation,
            };
            let ranges = self.detail_ranges(object.scale);
            let detail = PageDetail {
                radius: object.scale,
                focal_length_pixels: f64::from(self.focal_length_pixels),
                vertex_spacing_angle: ranges.vertex_spacing_angle(),
            };
            let finest_color = terrain
                .pages
                .as_ref()
                .and_then(|pages| pages.layer(TerrainLayer::Color))
                .map(|(_, finest)| finest);
            let surfaces = terrain
                .surfaces
                .as_deref()
                .filter(|_| self.noise_volume.index().is_some());
            let range = |tile: &SelectedTile| match (surfaces, finest_color) {
                (Some(_), Some(level)) if detail.magnifies(level, tile.nearest) => ShadingRange::Near,
                _ => ShadingRange::Far,
            };
            let draw_key = |tile: &SelectedTile| (range(tile), DetailRanges::grid_cells(tile.detail));
            let first_tile = self.selected_tiles.len();
            sphere.select(&ranges, &culler, &mut self.selected_tiles);
            let tiles = &mut self.selected_tiles[first_tile..];
            tiles.sort_by_key(draw_key);
            let mut run_start = first_tile;

            for run in tiles.chunk_by(|first, second| draw_key(first) == draw_key(second)) {
                self.terrain_draws.push(TerrainDraw {
                    terrain: TerrainHandle { index },
                    shape,
                    range: range(&run[0]),
                    grid_cells: DetailRanges::grid_cells(run[0].detail),
                    first_tile: run_start as u32,
                    tile_count: run.len() as u32,
                });
                run_start += run.len();
            }
        }
    }

    // Tile centers are placed in f64, relative to each frame's camera, so only offsets within a tile reach f32. Each
    // tile's pages are written beside it, and the pages tiles need are requested.
    fn write_tiles(&mut self, camera: &Camera) {
        self.tile_data.clear();
        self.tile_pages.clear();
        self.page_writer.begin(self.scene.terrains().len());
        let previous_camera_position = self.temporal.previous_camera_position();

        for draw in &self.terrain_draws {
            let terrain = self.scene.terrain(draw.terrain);
            let object = self.scene.object(terrain.object);
            let previous = self.temporal.previous_pose(terrain.object.index);
            let ranges = self.detail_ranges(object.scale);
            let detail = PageDetail {
                radius: object.scale,
                focal_length_pixels: f64::from(self.focal_length_pixels),
                vertex_spacing_angle: ranges.vertex_spacing_angle(),
            };
            let first_tile = draw.first_tile as usize;

            for selected in &self.selected_tiles[first_tile..first_tile + draw.tile_count as usize] {
                let tile = selected.tile;
                let direction = tile.cube_center().normalize();
                let center = object.position - camera.position + object.orientation * (direction * object.scale);
                let previous_center =
                    previous.position - previous_camera_position + previous.orientation * (direction * previous.scale);
                self.tile_data.push(TileData {
                    center: center.as_vec3(),
                    detail: selected.detail,
                    previous_center: previous_center.as_vec3(),
                    grid_cells: draw.grid_cells,
                    cube_center: tile.cube_center().as_vec3(),
                    morph_start: ranges.morph_start(selected.detail) as f32,
                    column_axis: tile.column_axis().as_vec3(),
                    morph_end: ranges.morph_end(selected.detail) as f32,
                    row_axis: tile.row_axis().as_vec3(),
                    half_size: tile.half_size() as f32,
                    x: tile.x,
                    y: tile.y,
                    depth: tile.depth,
                    layer_finest_levels: terrain.pages.as_ref().map_or(0, |pages| pages.layer_finest_levels()),
                });
                self.tile_pages
                    .push(terrain.pages.as_ref().map_or(TilePages::NONE, |pages| {
                        self.page_writer.write(draw.terrain.index, pages, selected, &detail)
                    }));
            }
        }

        self.page_writer
            .submit(self.scene.terrains().iter().map(|terrain| terrain.pages.as_deref()));
    }

    // Where the camera stands in each terrain's surfaces, computed in f64 in the body's frame. Surfaces are drawn only
    // once the noise they read is uploaded.
    fn write_surface_anchors(&mut self, camera: &Camera) {
        self.surface_anchor_data.clear();
        self.surface_anchor_slots.clear();
        let noise_volume = self.noise_volume.index();

        for terrain in self.scene.terrains() {
            let slot = terrain
                .surfaces
                .as_ref()
                .zip(noise_volume)
                .map(|(surfaces, noise_volume)| {
                    let object = self.scene.object(terrain.object);
                    let camera_in_body = object.orientation.inverse() * (camera.position - object.position);
                    self.surface_anchor_data
                        .push(surfaces.anchor_data(camera_in_body, noise_volume));
                    self.surface_anchor_data.len() - 1
                });
            self.surface_anchor_slots.push(slot);
        }
    }

    // A captured frame renders into an image of its own the video reads, then shows in the window through a copy.
    fn record_frame(&mut self, output: FrameOutput, captured: bool, clear_color: Vec3) -> anyhow::Result<Recording> {
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
        let window: Option<Subresource> = match output {
            FrameOutput::Window { image_index, .. } => Some(
                graph
                    .import_image(ImportedImage {
                        name: "swapchain",
                        image: self.swapchain.image(image_index),
                        view: self.swapchain.image_view(image_index),
                        extent: self.swapchain.extent(),
                        aspect: vk::ImageAspectFlags::COLOR,
                        initial: SWAPCHAIN_ACQUIRED,
                        final_state: SWAPCHAIN_PRESENTABLE,
                    })
                    .into(),
            ),
            FrameOutput::Offscreen => None,
        };
        let captured_frame = captured.then(|| {
            graph.create_image(GraphImageDescription {
                name: "captured frame",
                format: self.swapchain.format(),
                size: ImageSize::Output,
                mip_levels: 1,
            })
        });
        let output: Subresource = match (captured_frame, window) {
            (Some(frame), _) => frame.into(),
            (None, Some(window)) => window,
            (None, None) => graph
                .create_image(GraphImageDescription {
                    name: "offscreen output",
                    format: self.swapchain.format(),
                    size: ImageSize::Output,
                    mip_levels: 1,
                })
                .into(),
        };
        let video = captured_frame.map(|frame| VideoTarget { frame, window });
        let inputs = FrameInputs {
            output,
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
            video,
            projected_light_count: self.projected_light_data.len() as u32,
            lens_loaded: self.lens.as_ref().and_then(Lens::flare).is_some(),
            clear_color,
        };
        self.passes.add_to(&mut graph, &self.config, inputs);
        let compiled = graph.compile()?;
        self.graph_textures
            .register(bindless, compiled.graph(), compiled.created_images())?;
        let context = FrameContext {
            recorder: CommandRecorder::new(device, frame.command_buffer),
            frame,
            frame_slot: self.frame_index,
            textures: &self.graph_textures,
            temporal: &self.temporal,
            config: &self.config,
            scene: &self.scene,
            materials: &self.materials,
            lens: self.lens.as_ref().and_then(Lens::flare),
            focal_length_pixels: self.focal_length_pixels,
            terrain_draws: &self.terrain_draws,
            surface_anchor_slots: &self.surface_anchor_slots,
            descriptor_set: bindless.set(),
            user_interface: &self.user_interface,
            video: video.and(self.video.as_ref()),
            triangles: Cell::new(0),
        };
        compiled.execute(device.handle(), frame.command_buffer, &context)?;

        if let Some(timer) = &frame.timer {
            timer.end(device, frame.command_buffer);
        }

        let mut terrain_pages = TerrainPageStatistics::default();

        for pages in self
            .scene
            .terrains()
            .iter()
            .filter_map(|terrain| terrain.pages.as_ref())
        {
            terrain_pages.add(&pages.statistics());
        }

        self.statistics = FrameStatistics {
            terrain_pages,
            triangles: context.triangles.get(),
            tiles: self.selected_tiles.len(),
            deepest_depth: self.selected_tiles.iter().map(|selected| selected.tile.depth).max(),
            gpu_milliseconds: self.gpu_milliseconds,
        };
        self.user_interface.atlas_updates.clear();

        Ok(recording)
    }

    fn submit_offscreen(&mut self, recording: Recording) -> anyhow::Result<()> {
        let submitted = recording.submit(&[self.gpu.transfer.visibility_wait()], &[])?;
        self.frames[self.frame_index].submitted = Some(submitted);

        Ok(())
    }

    fn submit_and_present(&mut self, window: &Window, image_index: u32, recording: Recording) -> anyhow::Result<()> {
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
        // On Wayland, winit then holds `RedrawRequested` until the compositor asks for another frame, so a hidden
        // window stops drawing instead of blocking in the next present.
        window.pre_present_notify();

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
            frame.tile_pages = Frame::create_tile_pages_buffer(&self.gpu.memory, self.tile_capacity)?;
        }

        Ok(())
    }

    fn ensure_surface_anchor_capacity(&mut self) -> anyhow::Result<()> {
        let anchor_count = self.surface_anchor_data.len();

        if anchor_count <= self.surface_anchor_capacity {
            return Ok(());
        }

        self.surface_anchor_capacity = anchor_count.next_power_of_two();

        for frame in &mut self.frames {
            frame.surface_anchors =
                Frame::create_surface_anchor_buffer(&self.gpu.memory, self.surface_anchor_capacity)?;
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
        self.graph_textures
            .register(&self.gpu.bindless, &self.graph, &recreated)?;
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
