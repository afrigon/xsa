mod forward_resources;

pub(in crate::renderer) use forward_resources::ForwardResources;

use std::collections::HashMap;
use std::mem::offset_of;

use ash::vk;
use render_graph::{Attachment, GraphImageDescription, ImageSize, PassContext, PassDeclaration, PassError, RenderPass};

use crate::mesh::{GridMesh, GridVertex};
use crate::renderer::frame_context::FrameContext;
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::gpu_data::PushConstants;
use crate::renderer::quad_sphere::{COARSEST_DETAIL, DetailRanges};
use crate::renderer::{Shader, ShaderBinaries};
use crate::vulkan::{Buffer, GraphicsPipeline, GraphicsPipelineDescription};

const TRIANGLES_PER_CELL: u64 = 2;
pub(in crate::renderer) const HDR_FORMAT: vk::Format = vk::Format::R32G32B32A32_SFLOAT;
pub(in crate::renderer) const MOTION_FORMAT: vk::Format = vk::Format::R16G16_SFLOAT;
pub(in crate::renderer) const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;
const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const REVERSE_Z_FAR_DEPTH: f32 = 0.0;

pub(in crate::renderer) struct ForwardPass {
    pipelines: HashMap<Shader, GraphicsPipeline>,
    grids: GridMesh,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl ForwardPass {
    pub fn new(gpu: &mut GpuContext, binaries: &ShaderBinaries) -> anyhow::Result<ForwardPass> {
        let pipelines = ForwardPass::object_shaders()
            .map(|shader| {
                let pipeline = ForwardPass::create_pipeline(gpu, &binaries.shaders[shader.index()])?;
                Ok((shader, pipeline))
            })
            .collect::<anyhow::Result<_>>()?;
        let grids = GridMesh::new((COARSEST_DETAIL..=0).map(DetailRanges::grid_cells));
        let vertex_buffer = Buffer::with_data(
            &gpu.device,
            &mut gpu.allocator,
            "tile grid vertices",
            vk::BufferUsageFlags::VERTEX_BUFFER,
            &grids.vertices,
        )?;
        let index_buffer = Buffer::with_data(
            &gpu.device,
            &mut gpu.allocator,
            "tile grid indices",
            vk::BufferUsageFlags::INDEX_BUFFER,
            &grids.indices,
        )?;

        Ok(ForwardPass {
            pipelines,
            grids,
            vertex_buffer,
            index_buffer,
        })
    }

    pub unsafe fn destroy(&mut self, gpu: &mut GpuContext) {
        unsafe {
            for pipeline in self.pipelines.values_mut() {
                pipeline.destroy(&gpu.device);
            }

            self.vertex_buffer.destroy(&gpu.device, &mut gpu.allocator);
            self.index_buffer.destroy(&gpu.device, &mut gpu.allocator);
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

    // The skybox is drawn by its own pass.
    fn object_shaders() -> impl Iterator<Item = Shader> {
        Shader::ALL.iter().copied().filter(|shader| *shader != Shader::Skybox)
    }

    fn pipeline(&self, shader: Shader) -> &GraphicsPipeline {
        &self.pipelines[&shader]
    }

    fn record_objects(&self, frame: &FrameContext) {
        let recorder = &frame.recorder;
        recorder.set_wireframe(frame.render.wireframe);
        recorder.bind_mesh(&self.vertex_buffer, &self.index_buffer);

        for shader in ForwardPass::object_shaders() {
            let pipeline = self.pipeline(shader);
            let mut bound = false;

            for draw in frame.terrain_draws {
                let terrain = frame.scene.terrain(draw.terrain);
                let terrain_shader = frame
                    .render
                    .shader_override
                    .unwrap_or(frame.scene.material(terrain.material).shader());

                if terrain_shader != shader {
                    continue;
                }

                if !bound {
                    recorder.bind_graphics(pipeline, frame.descriptor_set);
                    bound = true;
                }

                frame.push_draw_constants(
                    pipeline,
                    terrain.object.index,
                    terrain.material.index(),
                    draw.first_tile,
                );
                recorder.draw_indexed(self.grids.indices(draw.grid_cells), draw.tile_count);
                let cells = u64::from(draw.grid_cells).pow(2);
                frame
                    .triangles
                    .set(frame.triangles.get() + cells * TRIANGLES_PER_CELL * u64::from(draw.tile_count));
            }
        }

        recorder.set_wireframe(false);
    }

    fn create_pipeline(gpu: &GpuContext, spirv: &[u8]) -> anyhow::Result<GraphicsPipeline> {
        let vertex_bindings = [vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<GridVertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)];
        let vertex_attributes = [vk::VertexInputAttributeDescription::default()
            .location(0)
            .binding(0)
            .format(vk::Format::R32G32_UINT)
            .offset(offset_of!(GridVertex, cell) as u32)];
        let description = GraphicsPipelineDescription {
            spirv,
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
        GraphicsPipeline::new(&gpu.device, &description)
    }
}

impl<'frame> RenderPass<FrameContext<'frame>> for ForwardPass {
    const NAME: &'static str = "forward";

    type Inputs = ();
    type Resources = ForwardResources;

    fn declare(&self, pass: &mut PassDeclaration, _inputs: ()) -> ForwardResources {
        let hdr = pass.create_image(ForwardPass::target("hdr color", HDR_FORMAT));
        let motion = pass.create_image(ForwardPass::target("motion vectors", MOTION_FORMAT));
        let depth = pass.create_image(ForwardPass::target("depth", DEPTH_FORMAT));
        pass.color_attachment(hdr, Attachment::ClearColor(CLEAR_COLOR));
        pass.color_attachment(motion, Attachment::ClearColor(CLEAR_COLOR));
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
