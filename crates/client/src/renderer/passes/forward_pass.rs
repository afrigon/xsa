mod forward_inputs;
mod forward_resources;

pub(in crate::renderer) use forward_inputs::ForwardInputs;
pub(in crate::renderer) use forward_resources::ForwardResources;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::mem::offset_of;

use ash::vk;
use memory_vulkan::Buffer;
use render_graph_vulkan::{
    Attachment, BufferUsage, GraphImageDescription, ImageSize, PassContext, PassDeclaration, PassError, RenderPass,
    Stage,
};
use resource_loader::mesh::MeshVertex;

use crate::config::RenderConfig;
use crate::mesh::{GridMesh, GridVertex};
use crate::renderer::forward_pipeline::ForwardPipeline;
use crate::renderer::frame_context::FrameContext;
use crate::renderer::geometry_variant::GeometryVariant;
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::gpu_data::PushConstants;
use crate::renderer::quad_sphere::{COARSEST_DETAIL, DetailRanges};
use crate::renderer::scatter::DRAW_ARGUMENTS_BYTES;
use crate::renderer::shading_range::ShadingRange;
use crate::renderer::static_buffer::StaticBuffer;
use crate::renderer::terrain_variant::TerrainVariant;
use crate::renderer::tile_shape::TileShape;
use crate::renderer::{GeometryFamily, Shader, ShaderBinaries};
use crate::vulkan::{GraphicsPipeline, GraphicsPipelineDescription};

const TRIANGLES_PER_CELL: u64 = 2;
pub(in crate::renderer) const HDR_FORMAT: vk::Format = vk::Format::R32G32B32A32_SFLOAT;
pub(in crate::renderer) const MOTION_FORMAT: vk::Format = vk::Format::R16G16_SFLOAT;
pub(in crate::renderer) const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;
const ZERO_MOTION: [f32; 4] = [0.0; 4];
const REVERSE_Z_FAR_DEPTH: f32 = 0.0;

pub(in crate::renderer) struct ForwardPass {
    programs: HashMap<Shader, Vec<u8>>,
    pipelines: HashMap<ForwardPipeline, GraphicsPipeline>,
    grids: GridMesh,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl ForwardPass {
    pub fn new(gpu: &GpuContext, binaries: &ShaderBinaries) -> anyhow::Result<ForwardPass> {
        let grids = GridMesh::new((COARSEST_DETAIL..=0).map(|detail| DetailRanges::grid_cells(0, detail)));
        // Every frame draws the grid, so it is in GPU memory before the first one.
        let vertex_buffer = StaticBuffer::new(
            gpu,
            "tile grid vertices",
            vk::BufferUsageFlags::VERTEX_BUFFER,
            &grids.vertices,
        )?;
        let index_buffer = StaticBuffer::new(
            gpu,
            "tile grid indices",
            vk::BufferUsageFlags::INDEX_BUFFER,
            &grids.indices,
        )?;
        let vertex_buffer = vertex_buffer.into_buffer()?;
        let index_buffer = index_buffer.into_buffer()?;

        Ok(ForwardPass {
            programs: binaries.forward.clone(),
            pipelines: HashMap::new(),
            grids,
            vertex_buffer,
            index_buffer,
        })
    }

    // Builds the pipelines `render`'s debug views draw with, the first time they are shown.
    pub fn ensure_pipelines(&mut self, gpu: &GpuContext, render: &RenderConfig) -> anyhow::Result<()> {
        for (shader, spirv) in &self.programs {
            for geometry in ForwardPass::geometry_variants(shader.family(), render) {
                let key = ForwardPipeline {
                    shader: *shader,
                    view: render.shader_override,
                    geometry,
                };

                if let Entry::Vacant(entry) = self.pipelines.entry(key) {
                    entry.insert(ForwardPass::create_pipeline(gpu, spirv, key)?);
                }
            }
        }

        Ok(())
    }

    // The family's variants `render`'s views select.
    fn geometry_variants(family: GeometryFamily, render: &RenderConfig) -> Vec<GeometryVariant> {
        match family {
            GeometryFamily::Terrain => TileShape::ALL
                .into_iter()
                .flat_map(|shape| {
                    ShadingRange::ALL.map(|range| {
                        GeometryVariant::Terrain(TerrainVariant {
                            shape,
                            range,
                            terrain_view: render.terrain_shader_override,
                            surface_view: render.surface_shader_override,
                        })
                    })
                })
                .collect(),
            GeometryFamily::Mesh => vec![GeometryVariant::Mesh],
            GeometryFamily::Sky => Vec::new(),
        }
    }

    fn target(name: &'static str, format: vk::Format) -> GraphImageDescription {
        GraphImageDescription {
            name,
            format,
            size: ImageSize::Output,
            mip_levels: 1,
        }
    }

    fn record_objects(&self, frame: &FrameContext) {
        let recorder = &frame.recorder;
        recorder.set_wireframe(frame.config.debug.wireframe);
        self.record_terrain(frame);
        self.record_meshes(frame);
        recorder.set_wireframe(false);
    }

    fn record_terrain(&self, frame: &FrameContext) {
        let recorder = &frame.recorder;
        recorder.bind_mesh(&self.vertex_buffer, &self.index_buffer);

        let drawn = self
            .pipelines
            .iter()
            .filter(|(key, _)| ForwardPass::shows_views(key, &frame.config.render));

        for (key, pipeline) in drawn {
            let GeometryVariant::Terrain(variant) = key.geometry else {
                continue;
            };
            let mut bound = false;

            for (index, draw) in frame.terrain_draws.iter().enumerate() {
                let terrain = frame.scene.terrain(draw.terrain);

                if draw.shape != variant.shape
                    || draw.range != variant.range
                    || frame.material(terrain.material).shader() != Some(key.shader)
                {
                    continue;
                }

                if !bound {
                    recorder.bind_graphics(pipeline, frame.descriptor_set);
                    bound = true;
                }

                frame.push_draw_constants(pipeline, terrain.material, frame.terrain_draw(index));
                recorder.draw_indexed(self.grids.indices(draw.grid_cells), draw.tile_count);
                let cells = u64::from(draw.grid_cells).pow(2);
                frame
                    .terrain_triangles
                    .set(frame.terrain_triangles.get() + cells * TRIANGLES_PER_CELL * u64::from(draw.tile_count));
            }
        }
    }

    // Each scatter submesh draws its model's instance list, as many as the cull pass counted into its arguments.
    fn record_meshes(&self, frame: &FrameContext) {
        let recorder = &frame.recorder;
        let arguments = frame.frame.scatter.arguments.handle();
        let drawn = self
            .pipelines
            .iter()
            .filter(|(key, _)| key.geometry == GeometryVariant::Mesh)
            .filter(|(key, _)| ForwardPass::shows_views(key, &frame.config.render));

        for (key, pipeline) in drawn {
            let mut bound = false;
            let mut bound_model = None;

            for (index, draw) in frame.scatter.mesh_draws.iter().enumerate() {
                if frame.material(draw.material).shader() != Some(key.shader) {
                    continue;
                }

                if !bound {
                    recorder.bind_graphics(pipeline, frame.descriptor_set);
                    bound = true;
                }

                if bound_model != Some(draw.model) {
                    let mesh = frame
                        .scene
                        .model(draw.model)
                        .mesh
                        .get()
                        .expect("drawn models have their mesh");
                    recorder.bind_mesh(mesh.vertices(), mesh.indices());
                    bound_model = Some(draw.model);
                }

                frame.push_draw_constants(pipeline, draw.material, frame.mesh_draw(index));
                recorder.draw_indexed_indirect(arguments, index as u64 * DRAW_ARGUMENTS_BYTES);
            }
        }
    }

    fn shows_views(key: &ForwardPipeline, render: &RenderConfig) -> bool {
        let shows_geometry_views = match key.geometry {
            GeometryVariant::Terrain(variant) => {
                variant.terrain_view == render.terrain_shader_override
                    && variant.surface_view == render.surface_shader_override
            }
            GeometryVariant::Mesh => true,
        };

        key.view == render.shader_override && shows_geometry_views
    }

    fn vertex_attribute(location: u32, format: vk::Format, offset: usize) -> vk::VertexInputAttributeDescription {
        vk::VertexInputAttributeDescription::default()
            .location(location)
            .binding(0)
            .format(format)
            .offset(offset as u32)
    }

    fn create_pipeline(gpu: &GpuContext, spirv: &[u8], key: ForwardPipeline) -> anyhow::Result<GraphicsPipeline> {
        let (stride, vertex_attributes) = match key.geometry {
            GeometryVariant::Terrain(_) => (
                size_of::<GridVertex>(),
                vec![ForwardPass::vertex_attribute(
                    0,
                    vk::Format::R32G32_UINT,
                    offset_of!(GridVertex, cell),
                )],
            ),
            GeometryVariant::Mesh => (
                size_of::<MeshVertex>(),
                vec![
                    ForwardPass::vertex_attribute(0, vk::Format::R32G32B32_SFLOAT, offset_of!(MeshVertex, position)),
                    ForwardPass::vertex_attribute(
                        1,
                        vk::Format::R16G16B16A16_SNORM,
                        offset_of!(MeshVertex, tangent_frame),
                    ),
                ],
            ),
        };
        let vertex_bindings = [vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(stride as u32)
            .input_rate(vk::VertexInputRate::VERTEX)];
        let description = GraphicsPipelineDescription {
            spirv,
            specialization_constants: &key.specialization_constants(),
            color_formats: &[HDR_FORMAT, MOTION_FORMAT],
            depth_format: Some(DEPTH_FORMAT),
            depth_compare_op: vk::CompareOp::GREATER,
            depth_write: true,
            cull_mode: vk::CullModeFlags::BACK,
            additive_blend: false,
            wireframe: true,
            vertex_bindings: &vertex_bindings,
            vertex_attributes: &vertex_attributes,
            push_constant_size: size_of::<PushConstants>() as u32,
            descriptor_set_layouts: &[gpu.bindless.layout()],
        };
        GraphicsPipeline::new(&gpu.memory, &gpu.device, &description)
    }
}

impl<'frame> RenderPass<FrameContext<'frame>> for ForwardPass {
    const NAME: &'static str = "forward";

    type Inputs = ForwardInputs;
    type Resources = ForwardResources;

    fn declare(&self, pass: &mut PassDeclaration, inputs: ForwardInputs) -> ForwardResources {
        if let Some(scatter) = inputs.scatter {
            pass.buffer(scatter.arguments, BufferUsage::IndirectRead);
            pass.buffer(scatter.instances, BufferUsage::StorageRead(Stage::Vertex));
        }

        let hdr = pass.create_image(ForwardPass::target("hdr color", HDR_FORMAT));
        let motion = pass.create_image(ForwardPass::target("motion vectors", MOTION_FORMAT));
        let depth = pass.create_image(ForwardPass::target("depth", DEPTH_FORMAT));
        pass.color_attachment(hdr, Attachment::ClearColor(inputs.clear_color));
        pass.color_attachment(motion, Attachment::ClearColor(ZERO_MOTION));
        pass.depth_attachment(depth, Attachment::ClearDepth(REVERSE_Z_FAR_DEPTH));

        ForwardResources { hdr, motion, depth }
    }

    fn record(
        &mut self,
        frame: &FrameContext<'frame>,
        pass: &PassContext,
        _resources: &ForwardResources,
    ) -> Result<(), PassError> {
        pass.begin_rendering();
        self.record_objects(frame);
        pass.end_rendering();

        Ok(())
    }
}
