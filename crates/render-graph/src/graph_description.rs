use crate::compiled_graph::CompiledGraph;
use crate::frame_declaration::FrameDeclaration;
use crate::{BufferUsageDescription, ImageUsageDescription, PassDescription, StepDescription};

/// A compiled frame as plain data, for inspecting how passes connect.
#[derive(Clone, Debug)]
pub struct GraphDescription {
    pub passes: Vec<PassDescription>,
}

impl GraphDescription {
    pub(crate) fn new(declaration: &FrameDeclaration, compiled: &CompiledGraph) -> GraphDescription {
        let passes = declaration
            .passes
            .iter()
            .enumerate()
            .map(|(index, pass)| {
                let compiled_pass = compiled.passes.iter().find(|compiled| compiled.declaration == index);
                let steps = (0..pass.step_count)
                    .map(|step| StepDescription {
                        image_usages: declaration
                            .image_usages
                            .iter()
                            .filter(|usage| usage.pass == index && usage.step == step)
                            .map(|usage| ImageUsageDescription {
                                image: declaration.images[usage.image].name(),
                                level: usage.level,
                                usage: usage.usage,
                            })
                            .collect(),
                        buffer_usages: declaration
                            .buffer_usages
                            .iter()
                            .filter(|usage| usage.pass == index && usage.step == step)
                            .map(|usage| BufferUsageDescription {
                                buffer: declaration.buffers[usage.buffer].name,
                                usage: usage.usage,
                            })
                            .collect(),
                        barriers: compiled_pass.map_or(0, |compiled| {
                            let step = &compiled.steps[step as usize];
                            step.image_barriers.len() + step.buffer_barriers.len()
                        }),
                    })
                    .collect();

                PassDescription {
                    name: pass.name,
                    culled: compiled_pass.is_none(),
                    steps,
                }
            })
            .collect();

        GraphDescription { passes }
    }
}
