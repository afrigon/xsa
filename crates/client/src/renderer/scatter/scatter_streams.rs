use std::collections::HashSet;

use ash::vk;
use glam::{DVec3, Vec3};
use resource_loader::mesh::BoundingBox;
use resource_loader::utils::HalfFloat;
use resource_vulkan::Mesh;
use xsa_core::scatter::{ScatterInstance, ScatterOrientation};
use xsa_core::terrain::QuadSphereTile;

use super::model_check::ModelCheck;
use super::scatter_generator::ScatterGenerator;
use super::scatter_list::ScatterList;
use super::scatter_mesh_draw::ScatterMeshDraw;
use super::scatter_page::ScatterPage;
use super::scatter_request::ScatterRequest;
use super::scatter_stream::ScatterStream;
use super::{NO_LIST, ScatterFrame, ScatterView};
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::gpu_data::{
    ScatterDrawData, ScatterInstanceData, ScatterModelData, ScatterPageData, ScatterUseData,
};
use crate::renderer::static_buffer::StaticBuffer;
use crate::renderer::{BodyScatter, Model, ModelHandle, Scene};

const TRIANGLE_INDICES: u64 = 3;
const HALF_BITS: u32 = 16;

// Keeps each body scatter's pages within range of the camera: placed on a worker, uploaded once, dropped once out of
// range. Every frame, it lists what the frame culls and draws.
pub(in crate::renderer) struct ScatterStreams {
    // By body scatter.
    streams: Vec<ScatterStream>,
    // By model.
    checks: Vec<ModelCheck>,
    generator: ScatterGenerator,
    frame: ScatterFrame,
}

impl ScatterStreams {
    pub fn new() -> anyhow::Result<ScatterStreams> {
        Ok(ScatterStreams {
            streams: Vec::new(),
            checks: Vec::new(),
            generator: ScatterGenerator::new()?,
            frame: ScatterFrame::default(),
        })
    }

    pub fn frame(&self) -> &ScatterFrame {
        &self.frame
    }

    pub fn update(&mut self, gpu: &GpuContext, view: &ScatterView) -> anyhow::Result<()> {
        let scene = view.scene;
        self.streams.resize_with(scene.scatters().len(), ScatterStream::default);
        self.checks.resize(scene.models().len(), ModelCheck::Pending);
        self.receive(gpu, scene)?;

        for (index, scatter) in scene.scatters().iter().enumerate() {
            self.stream(index, scatter, view)?;
        }

        self.check_models(scene);
        self.build_frame(view);

        Ok(())
    }

    fn receive(&mut self, gpu: &GpuContext, scene: &Scene) -> anyhow::Result<()> {
        let results: Vec<_> = self.generator.results().collect();

        for result in results {
            let stream = &mut self.streams[result.scatter];

            if !stream.requested.remove(&result.page) {
                continue;
            }

            let model_count = scene.scatters()[result.scatter].models.len();
            let page = ScatterStreams::upload(gpu, &result.instances, model_count)?;
            stream.pages.insert(result.page, page);
        }

        Ok(())
    }

    fn upload(gpu: &GpuContext, instances: &[ScatterInstance], model_count: usize) -> anyhow::Result<ScatterPage> {
        let mut model_counts = vec![0; model_count];

        for instance in instances {
            model_counts[instance.model] += 1;
        }

        let data: Vec<ScatterInstanceData> = instances.iter().map(ScatterStreams::instance_data).collect();
        let buffer = (!data.is_empty())
            .then(|| {
                StaticBuffer::new(
                    gpu,
                    "scatter page",
                    vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
                    &data,
                )
            })
            .transpose()?;

        Ok(ScatterPage {
            uploaded: buffer.is_none(),
            buffer,
            instance_count: data.len() as u32,
            model_counts,
        })
    }

    fn instance_data(instance: &ScatterInstance) -> ScatterInstanceData {
        ScatterInstanceData {
            offset: instance.offset.as_vec2().to_array(),
            yaw_and_scale: ScatterStreams::halves(instance.yaw, instance.scale),
            tilt_and_heading: ScatterStreams::halves(instance.tilt, instance.tilt_heading),
            sink: instance.sink as f32,
            model: instance.model as u32,
        }
    }

    // Two half floats in one word, the first low.
    fn halves(low: f64, high: f64) -> u32 {
        u32::from(HalfFloat::from_f32(low as f32).bits)
            | (u32::from(HalfFloat::from_f32(high as f32).bits) << HALF_BITS)
    }

    // The pages within the scatter's range of the camera, once the body's heights are loaded: the ground they stand on.
    fn stream(&mut self, index: usize, scatter: &BodyScatter, view: &ScatterView) -> anyhow::Result<()> {
        let wanted = ScatterStreams::wanted_pages(scatter, view);
        let stream = &mut self.streams[index];
        stream.pages.retain(|page, _| wanted.contains(page));
        stream.requested.retain(|page| wanted.contains(page));

        for page in wanted {
            if stream.pages.contains_key(&page) || !stream.requested.insert(page) {
                continue;
            }

            self.generator.request(ScatterRequest {
                scatter: index,
                page,
                placement: scatter.placement.clone(),
            });
        }

        for page in stream.pages.values_mut() {
            if let Some(buffer) = &mut page.buffer
                && !page.uploaded
            {
                page.uploaded = buffer.poll()?;
            }
        }

        Ok(())
    }

    fn wanted_pages(scatter: &BodyScatter, view: &ScatterView) -> HashSet<QuadSphereTile> {
        if !view.enabled {
            return HashSet::new();
        }

        let terrain = view.scene.terrain(scatter.terrain);
        let Some(maps) = terrain.maps.as_deref() else {
            return HashSet::new();
        };
        let object = view.scene.object(terrain.object);
        let camera = object.orientation.inverse() * (view.camera - object.position);
        let direction = camera.normalize();
        let Some(height) = maps.height_at(direction) else {
            return HashSet::new();
        };
        let range = scatter.placement.scatter.range;
        let altitude = camera.length() - object.scale - height * view.height_scale;

        if altitude > range {
            return HashSet::new();
        }

        scatter
            .placement
            .grid
            .pages_within(direction, range / object.scale)
            .into_iter()
            .collect()
    }

    // Logged once per model: a node without a material, or a material for a node the mesh lacks.
    fn check_models(&mut self, scene: &Scene) {
        for (index, model) in scene.models().iter().enumerate() {
            if self.checks[index] != ModelCheck::Pending {
                continue;
            }

            let Some(mesh) = model.mesh.get() else {
                continue;
            };

            self.checks[index] = match ScatterStreams::mismatch(model, mesh) {
                Some(mismatch) => {
                    tracing::error!("model {}: {mismatch}; it is not drawn", model.name);
                    ModelCheck::Mismatched
                }
                None => ModelCheck::Matched,
            };
        }
    }

    fn mismatch(model: &Model, mesh: &Mesh) -> Option<String> {
        let unmatched_node = mesh
            .submeshes()
            .iter()
            .find(|submesh| !model.materials.iter().any(|material| material.node == submesh.node));

        if let Some(submesh) = unmatched_node {
            return Some(format!("its mesh's node `{}` has no material", submesh.node));
        }

        model
            .materials
            .iter()
            .find(|material| !mesh.submeshes().iter().any(|submesh| submesh.node == material.node))
            .map(|material| format!("its mesh has no node `{}`", material.node))
    }

    // A model draws once its mesh matches its materials, all in GPU memory.
    fn is_drawable(&self, view: &ScatterView, handle: ModelHandle) -> bool {
        let model = view.scene.model(handle);

        self.checks[handle.index] == ModelCheck::Matched
            && model.mesh.get().is_some()
            && model
                .materials
                .iter()
                .all(|material| view.materials[material.material.index].device_address().is_some())
    }

    fn build_frame(&mut self, view: &ScatterView) {
        let scene = view.scene;
        let mut frame = std::mem::take(&mut self.frame);
        frame.clear();
        let lists = self.build_lists(view, &mut frame);

        for (index, scatter) in scene.scatters().iter().enumerate() {
            let stream = &self.streams[index];

            if !stream.pages.values().any(ScatterPage::is_drawn) {
                continue;
            }

            let scatter_use = frame.uses.len() as u32;
            frame
                .uses
                .push(ScatterStreams::use_data(scatter, view, frame.models.len() as u32));

            for (place, &handle) in scatter.models.iter().enumerate() {
                let mesh = scene.model(handle).mesh.get();
                let list = lists[handle.index];
                let bounding_box = mesh.map(Mesh::bounding_box);
                frame.models.push(ScatterModelData {
                    box_center: bounding_box.map_or(Vec3::ZERO, BoundingBox::center),
                    list: list.map_or(NO_LIST, |list| list as u32),
                    box_half_extent: bounding_box.map_or(Vec3::ZERO, BoundingBox::half_extent),
                    first_instance: list.map_or(0, |list| frame.lists[list].first_instance),
                    orientation: match scatter.placement.scatter.models[place].orientation {
                        ScatterOrientation::Ground => 0,
                        ScatterOrientation::Up => 1,
                    },
                    padding: [0; 3],
                });
            }

            for (page, resident) in stream.pages.iter().filter(|(_, page)| page.is_drawn()) {
                frame
                    .pages
                    .push(ScatterStreams::page_data(scatter, view, *page, resident, scatter_use));
            }
        }

        self.frame = frame;
    }

    // An instance list for each drawable model with instances in resident pages, sized for all of them, and the draws
    // of its submeshes. Returns each model's list.
    fn build_lists(&self, view: &ScatterView, frame: &mut ScatterFrame) -> Vec<Option<usize>> {
        let scene = view.scene;
        let mut totals = vec![0_u32; scene.models().len()];

        for (index, scatter) in scene.scatters().iter().enumerate() {
            for page in self.streams[index].pages.values().filter(|page| page.is_drawn()) {
                for (place, count) in page.model_counts.iter().enumerate() {
                    totals[scatter.models[place].index] += count;
                }
            }
        }

        let mut lists = vec![None; totals.len()];

        for (index, &total) in totals.iter().enumerate() {
            let handle = ModelHandle { index };

            if total == 0 || !self.is_drawable(view, handle) {
                continue;
            }

            let model = scene.model(handle);
            let mesh = model.mesh.get().expect("drawable models have their mesh");
            let list = frame.lists.len();
            lists[index] = Some(list);
            frame.lists.push(ScatterList {
                first_instance: frame.instance_count,
                triangles_per_instance: mesh
                    .submeshes()
                    .iter()
                    .map(|submesh| u64::from(submesh.index_count) / TRIANGLE_INDICES)
                    .sum(),
            });
            frame.instance_count += total;

            for (submesh_index, submesh) in mesh.submeshes().iter().enumerate() {
                let material = model
                    .materials
                    .iter()
                    .find(|material| material.node == submesh.node)
                    .expect("matched models have a material per node");
                frame.draws.push(ScatterDrawData {
                    list: list as u32,
                    index_count: submesh.index_count,
                    first_index: submesh.first_index,
                    padding: 0,
                });
                frame.mesh_draws.push(ScatterMeshDraw {
                    model: handle,
                    submesh: submesh_index,
                    material: material.material,
                });
            }
        }

        lists
    }

    // The horizon is measured in f64: near the ground, its square is a few millionths.
    fn use_data(scatter: &BodyScatter, view: &ScatterView, first_model: u32) -> ScatterUseData {
        let terrain = view.scene.terrain(scatter.terrain);
        let object = view.scene.object(terrain.object);
        let lowest = terrain
            .maps
            .as_deref()
            .and_then(|maps| maps.heights())
            .map_or(0.0, |heights| heights.level().minimum() * view.height_scale);
        let lowest_radius = object.scale + lowest;
        let camera_from_center = (view.camera - object.position) / lowest_radius;

        ScatterUseData {
            object_index: terrain.object.index as u32,
            terrain_index: scatter.terrain.index as u32,
            first_model,
            range: scatter.placement.scatter.range as f32,
            tint: scatter.tint,
            lowest_radius: lowest_radius as f32,
            camera_from_center: camera_from_center.as_vec3(),
            horizon_squared: (camera_from_center.length_squared() - 1.0) as f32,
            ground_albedo: scatter.ground_albedo,
            padding: 0,
        }
    }

    // Centers are placed in f64, relative to each frame's camera, as tiles are.
    fn page_data(
        scatter: &BodyScatter,
        view: &ScatterView,
        page: QuadSphereTile,
        resident: &ScatterPage,
        scatter_use: u32,
    ) -> ScatterPageData {
        let terrain = view.scene.terrain(scatter.terrain);
        let object = view.scene.object(terrain.object);
        let previous = view.temporal.previous_pose(terrain.object.index);
        let direction: DVec3 = page.cube_center().normalize();
        let center = object.position - view.camera + object.orientation * (direction * object.scale);
        let previous_center = previous.position - view.temporal.previous_camera_position()
            + previous.orientation * (direction * previous.scale);

        ScatterPageData {
            center: center.as_vec3(),
            instance_count: resident.instance_count,
            previous_center: previous_center.as_vec3(),
            scatter_use,
            cube_center: page.cube_center().as_vec3(),
            half_size: page.half_size() as f32,
            column_axis: page.column_axis().as_vec3(),
            x: page.x,
            row_axis: page.row_axis().as_vec3(),
            y: page.y,
            instances: resident
                .buffer
                .as_ref()
                .expect("drawn pages have instances")
                .device_address(),
            depth: page.depth,
            face: page.face.index() as u32,
        }
    }
}
