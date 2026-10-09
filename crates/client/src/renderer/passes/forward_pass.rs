mod forward_resources;

pub(in crate::renderer) use forward_resources::ForwardResources;

use std::mem::offset_of;

use ash::vk;
use render_graph::{Attachment, GraphImageDescription, ImageSize, PassContext, PassDeclaration, PassError, RenderPass};

use crate::mesh::{Mesh, Vertex};
use crate::renderer::frame_context::FrameContext;
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::gpu_data::PushConstants;
use crate::renderer::{Shader, ShaderBinaries};
use crate::vulkan::{Buffer, GraphicsPipeline, GraphicsPipelineDescription};

const SPHERE_SUBDIVISIONS: u32 = 64;
const TRIANGLE_VERTICES: u32 = 3;
const HDR_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;
const MOTION_FORMAT: vk::Format = vk::Format::R16G16_SFLOAT;
const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;
const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const REVERSE_Z_FAR_DEPTH: f32 = 0.0;

pub(in crate::renderer) struct ForwardPass {
    pipelines: Vec<GraphicsPipeline>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
}

impl ForwardPass {
    pub fn new(gpu: &mut GpuContext, binaries: &ShaderBinaries) -> anyhow::Result<ForwardPass> {
        let pipelines = Shader::ALL
            .iter()
            .map(|shader| ForwardPass::create_pipeline(gpu, *shader, &binaries.shaders[shader.index()]))
            .collect::<anyhow::Result<_>>()?;
        let sphere = Mesh::cube_sphere(1.0, SPHERE_SUBDIVISIONS);
        let vertex_buffer = Buffer::with_data(
            &gpu.device,
            &mut gpu.allocator,
            "sphere vertices",
            vk::BufferUsageFlags::VERTEX_BUFFER,
            &sphere.vertices,
        )?;
        let index_buffer = Buffer::with_data(
            &gpu.device,
            &mut gpu.allocator,
            "sphere indices",
            vk::BufferUsageFlags::INDEX_BUFFER,
            &sphere.indices,
        )?;

        Ok(ForwardPass {
            pipelines,
            vertex_buffer,
            index_buffer,
            index_count: sphere.indices.len() as u32,
        })
    }

    pub unsafe fn destroy(&mut self, gpu: &mut GpuContext) {
        unsafe {
            for pipeline in &mut self.pipelines {
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

    fn pipeline(&self, shader: Shader) -> &GraphicsPipeline {
        &self.pipelines[shader.index()]
    }

    fn record_objects(&self, frame: &FrameContext) {
        let recorder = &frame.recorder;
        recorder.set_wireframe(frame.render.wireframe);
        recorder.bind_mesh(&self.vertex_buffer, &self.index_buffer);

        for shader in Shader::ALL.iter().copied().filter(|shader| *shader != Shader::Skybox) {
            let pipeline = self.pipeline(shader);
            let mut bound = false;

            for handle in frame.visible_objects {
                let object = &frame.scene.objects()[handle.index];
                let object_shader = frame
                    .render
                    .shader_override
                    .unwrap_or(frame.scene.material(object.material).shader());

                if object_shader != shader {
                    continue;
                }

                if !bound {
                    recorder.bind_graphics(pipeline, frame.descriptor_set);
                    bound = true;
                }

                frame.push_draw_constants(pipeline, handle.index, object.material.index());
                recorder.draw_indexed(self.index_count);
                frame
                    .triangles
                    .set(frame.triangles.get() + u64::from(self.index_count / TRIANGLE_VERTICES));
            }
        }

        recorder.set_wireframe(false);
    }

    fn record_skybox(&self, frame: &FrameContext) {
        let Some(skybox) = frame.scene.skybox else {
            return;
        };
        let pipeline = self.pipeline(Shader::Skybox);
        frame.recorder.bind_graphics(pipeline, frame.descriptor_set);
        frame.push_draw_constants(pipeline, 0, skybox.index());
        frame.recorder.draw_fullscreen();
    }

    fn create_pipeline(gpu: &GpuContext, shader: Shader, spirv: &[u8]) -> anyhow::Result<GraphicsPipeline> {
        let vertex_bindings = [vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)];
        let vertex_attributes = [
            vk::VertexInputAttributeDescription::default()
                .location(0)
                .binding(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex, position) as u32),
            vk::VertexInputAttributeDescription::default()
                .location(1)
                .binding(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex, normal) as u32),
        ];
        let mesh_description = GraphicsPipelineDescription {
            spirv,
            color_formats: &[HDR_FORMAT, MOTION_FORMAT],
            depth_format: Some(DEPTH_FORMAT),
            depth_compare_op: vk::CompareOp::GREATER,
            depth_write: true,
            cull_mode: vk::CullModeFlags::BACK,
            additive_blend: false,
            vertex_bindings: &vertex_bindings,
            vertex_attributes: &vertex_attributes,
            push_constant_size: size_of::<PushConstants>() as u32,
            descriptor_set_layouts: &[gpu.bindless.layout()],
        };
        let description = match shader {
            Shader::Skybox => GraphicsPipelineDescription {
                depth_compare_op: vk::CompareOp::GREATER_OR_EQUAL,
                depth_write: false,
                cull_mode: vk::CullModeFlags::NONE,
                vertex_bindings: &[],
                vertex_attributes: &[],
                ..mesh_description
            },
            _ => mesh_description,
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
        self.record_skybox(frame);
        pass.end_rendering();

        Ok(())
    }
}
