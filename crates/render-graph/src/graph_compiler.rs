use ash::vk;

use crate::access::Access;
use crate::barrier_source::BarrierSource;
use crate::compiled_attachment::CompiledAttachment;
use crate::compiled_barrier::CompiledBarrier;
use crate::compiled_buffer_barrier::CompiledBufferBarrier;
use crate::compiled_graph::CompiledGraph;
use crate::declared_image::DeclaredImage;
use crate::end_state::EndState;
use crate::frame_declaration::FrameDeclaration;
use crate::{ImageState, RenderGraphError, ResourceUsage};

pub(crate) const MAX_COLOR_ATTACHMENTS: usize = 8;

struct SubresourceAccess {
    pass: usize,
    step: u32,
    usage: usize,
    access: Access,
    reads: bool,
    writes: bool,
}

// Consecutive accesses one barrier covers: a single write, or reads sharing a layout.
struct AccessGroup {
    pass: usize,
    step: u32,
    last_pass: usize,
    last_step: u32,
    access: Access,
    reads: bool,
    writes: bool,
    usages: Vec<usize>,
}

impl AccessGroup {
    fn new(first: &SubresourceAccess) -> AccessGroup {
        AccessGroup {
            pass: first.pass,
            step: first.step,
            last_pass: first.pass,
            last_step: first.step,
            access: first.access,
            reads: first.reads,
            writes: first.writes,
            usages: vec![first.usage],
        }
    }

    fn merges(&self, next: &SubresourceAccess) -> bool {
        !self.writes && !next.writes && self.access.layout == next.access.layout
    }

    fn merge(&mut self, next: &SubresourceAccess) {
        self.access.stages |= next.access.stages;
        self.access.access |= next.access.access;
        self.last_pass = next.pass;
        self.last_step = next.step;
        self.usages.push(next.usage);
    }

    fn state(&self) -> ImageState {
        ImageState {
            stages: self.access.stages,
            access: self.access.write_access(),
            layout: self.access.layout,
        }
    }
}

// Turns a frame's declarations into the passes that run and the barriers between their steps. Pure: the same
// declarations always compile to the same graph.
pub(crate) struct GraphCompiler<'declaration> {
    declaration: &'declaration FrameDeclaration,
    live_passes: Vec<usize>,
    positions: Vec<Option<usize>>,
}

impl<'declaration> GraphCompiler<'declaration> {
    pub fn new(declaration: &'declaration FrameDeclaration) -> GraphCompiler<'declaration> {
        let mut compiler = GraphCompiler {
            declaration,
            live_passes: Vec::new(),
            positions: vec![None; declaration.passes.len()],
        };
        compiler.cull();

        compiler
    }

    pub fn compile(&self) -> Result<CompiledGraph, RenderGraphError> {
        self.validate_levels()?;
        let usage_flags = self.usage_flags();
        let mut compiled = CompiledGraph::new(self.declaration, &self.live_passes, usage_flags.clone());
        let mut store_ops = vec![vk::AttachmentStoreOp::STORE; self.declaration.image_usages.len()];

        for (index, image) in self.declaration.images.iter().enumerate() {
            let sampled_layout = Access::sampled_layout(usage_flags[index]);

            for level in 0..image.level_count() {
                let accesses = self.image_accesses(index, level, sampled_layout);
                let groups = self.group(image.name(), &accesses)?;

                if let (DeclaredImage::Transient(_), Some(first)) = (image, groups.first())
                    && first.reads
                {
                    return Err(self.error(first.pass, format!("reads {} before any pass writes it", image.name())));
                }

                self.place_image_barriers(&mut compiled, index, image, level, &groups);

                if let Some(last) = groups.last() {
                    if !image.persists() {
                        for usage in &last.usages {
                            store_ops[*usage] = vk::AttachmentStoreOp::DONT_CARE;
                        }
                    }

                    if !matches!(image, DeclaredImage::Imported { .. }) {
                        compiled.end_states.push(EndState {
                            image: index,
                            level,
                            state: last.state(),
                        });
                    }
                }
            }
        }

        for (index, buffer) in self.declaration.buffers.iter().enumerate() {
            let accesses = self.buffer_accesses(index);
            let groups = self.group(buffer.name, &accesses)?;
            let mut source = ImageState {
                stages: buffer.initial.stages,
                access: buffer.initial.access,
                layout: vk::ImageLayout::UNDEFINED,
            };

            for group in &groups {
                compiled.passes[group.pass].steps[group.step as usize]
                    .buffer_barriers
                    .push(CompiledBufferBarrier {
                        buffer: index,
                        source_stages: source.stages,
                        source_access: source.access,
                        destination_stages: group.access.stages,
                        destination_access: group.access.access,
                    });
                source = group.state();
            }

            compiled.final_buffer_barriers.push(CompiledBufferBarrier {
                buffer: index,
                source_stages: source.stages,
                source_access: source.access,
                destination_stages: buffer.final_state.stages,
                destination_access: buffer.final_state.access,
            });
        }

        self.place_attachments(&mut compiled, &store_ops, &usage_flags)?;
        compiled.merge_levels();

        Ok(compiled)
    }

    // A pass runs when it writes something kept after the frame, or something a running pass reads.
    fn cull(&mut self) {
        let declaration = self.declaration;
        let mut live_images = vec![false; declaration.images.len()];

        for pass in (0..declaration.passes.len()).rev() {
            let image_usages = || declaration.image_usages.iter().filter(move |usage| usage.pass == pass);
            let writes_buffer = declaration
                .buffer_usages
                .iter()
                .any(|usage| usage.pass == pass && usage.usage.writes());
            let writes_needed_image = image_usages().any(|usage| {
                usage.usage.writes() && (declaration.images[usage.image].persists() || live_images[usage.image])
            });

            if !declaration.passes[pass].keep && !writes_buffer && !writes_needed_image {
                continue;
            }

            for usage in image_usages().filter(|usage| usage.usage.reads()) {
                live_images[usage.image] = true;
            }

            self.live_passes.push(pass);
        }

        self.live_passes.reverse();

        for (position, pass) in self.live_passes.iter().enumerate() {
            self.positions[*pass] = Some(position);
        }
    }

    fn validate_levels(&self) -> Result<(), RenderGraphError> {
        for usage in &self.declaration.image_usages {
            let image = &self.declaration.images[usage.image];

            if let (Some(position), Some(level)) = (self.positions[usage.pass], usage.level)
                && level >= image.level_count()
            {
                return Err(self.error(
                    position,
                    format!(
                        "uses level {level} of {}, which has {}",
                        image.name(),
                        image.level_count()
                    ),
                ));
            }
        }

        Ok(())
    }

    // History images share their flags: the pair swaps roles every frame.
    fn usage_flags(&self) -> Vec<vk::ImageUsageFlags> {
        let declaration = self.declaration;
        let mut flags = vec![vk::ImageUsageFlags::empty(); declaration.images.len()];

        for usage in declaration
            .image_usages
            .iter()
            .filter(|usage| self.positions[usage.pass].is_some())
        {
            flags[usage.image] |= usage.usage.image_usage();
        }

        let history_flags: Vec<_> = declaration
            .images
            .iter()
            .enumerate()
            .filter_map(|(index, image)| match image {
                DeclaredImage::History { id, .. } => Some((*id, flags[index])),
                _ => None,
            })
            .collect();

        for (index, image) in declaration.images.iter().enumerate() {
            if let DeclaredImage::History { id, .. } = image {
                flags[index] = history_flags
                    .iter()
                    .filter(|(other, _)| other == id)
                    .fold(vk::ImageUsageFlags::empty(), |union, (_, other_flags)| {
                        union | *other_flags
                    });
            }
        }

        flags
    }

    fn image_accesses(&self, image: usize, level: u32, sampled_layout: vk::ImageLayout) -> Vec<SubresourceAccess> {
        self.declaration
            .image_usages
            .iter()
            .enumerate()
            .filter(|(_, usage)| usage.image == image && usage.level.is_none_or(|used| used == level))
            .filter_map(|(index, usage)| {
                self.positions[usage.pass].map(|pass| SubresourceAccess {
                    pass,
                    step: usage.step,
                    usage: index,
                    access: usage.usage.access(sampled_layout),
                    reads: usage.usage.reads(),
                    writes: usage.usage.writes(),
                })
            })
            .collect()
    }

    fn buffer_accesses(&self, buffer: usize) -> Vec<SubresourceAccess> {
        self.declaration
            .buffer_usages
            .iter()
            .enumerate()
            .filter(|(_, usage)| usage.buffer == buffer)
            .filter_map(|(index, usage)| {
                self.positions[usage.pass].map(|pass| SubresourceAccess {
                    pass,
                    step: usage.step,
                    usage: index,
                    access: usage.usage.access(),
                    reads: usage.usage.reads(),
                    writes: usage.usage.writes(),
                })
            })
            .collect()
    }

    fn group(&self, name: &str, accesses: &[SubresourceAccess]) -> Result<Vec<AccessGroup>, RenderGraphError> {
        let mut groups: Vec<AccessGroup> = Vec::new();

        for access in accesses {
            if let Some(group) = groups.last_mut() {
                if group.merges(access) {
                    group.merge(access);
                    continue;
                }

                if group.last_pass == access.pass && group.last_step == access.step {
                    return Err(self.error(
                        access.pass,
                        format!("uses {name} twice in one step with conflicting usages"),
                    ));
                }
            }

            groups.push(AccessGroup::new(access));
        }

        Ok(groups)
    }

    fn place_image_barriers(
        &self,
        compiled: &mut CompiledGraph,
        index: usize,
        image: &DeclaredImage,
        level: u32,
        groups: &[AccessGroup],
    ) {
        let mut previous: Option<&AccessGroup> = None;

        for group in groups {
            let source = match (previous, image) {
                (Some(previous), _) => BarrierSource::Known(previous.state()),
                (None, DeclaredImage::Transient(_)) => BarrierSource::PreviousFrame { discard: true },
                (None, DeclaredImage::History { .. }) => BarrierSource::PreviousFrame { discard: !group.reads },
                (None, DeclaredImage::Imported { initial, .. }) => BarrierSource::Known(*initial),
            };
            compiled.passes[group.pass].steps[group.step as usize]
                .image_barriers
                .push(CompiledBarrier {
                    image: index,
                    base_level: level,
                    level_count: 1,
                    source,
                    destination_stages: group.access.stages,
                    destination_access: group.access.access,
                    new_layout: group.access.layout,
                });
            previous = Some(group);
        }

        if let DeclaredImage::Imported {
            initial, final_state, ..
        } = image
        {
            let source = previous.map_or(*initial, AccessGroup::state);
            compiled.final_image_barriers.push(CompiledBarrier {
                image: index,
                base_level: level,
                level_count: 1,
                source: BarrierSource::Known(source),
                destination_stages: final_state.stages,
                destination_access: final_state.access,
                new_layout: final_state.layout,
            });
        }
    }

    fn place_attachments(
        &self,
        compiled: &mut CompiledGraph,
        store_ops: &[vk::AttachmentStoreOp],
        usage_flags: &[vk::ImageUsageFlags],
    ) -> Result<(), RenderGraphError> {
        for (index, usage) in self.declaration.image_usages.iter().enumerate() {
            let (Some(pass), Some(attachment)) = (self.positions[usage.pass], usage.usage.attachment()) else {
                continue;
            };
            let image = &self.declaration.images[usage.image];
            let level = match usage.level {
                Some(level) => level,
                None if image.level_count() == 1 => 0,
                None => {
                    return Err(self.error(
                        pass,
                        format!(
                            "renders to {}, which has several levels, without naming one",
                            image.name()
                        ),
                    ));
                }
            };
            let depth = matches!(usage.usage, ResourceUsage::DepthAttachment(_));
            let step = &mut compiled.passes[pass].steps[usage.step as usize];
            let color_count = step.attachments.iter().filter(|existing| !existing.depth).count();

            if depth && step.attachments.iter().any(|existing| existing.depth) {
                return Err(self.error(pass, "has several depth attachments in one step".to_string()));
            }

            if !depth && color_count == MAX_COLOR_ATTACHMENTS {
                return Err(self.error(
                    pass,
                    format!("has more than {MAX_COLOR_ATTACHMENTS} color attachments in one step"),
                ));
            }

            step.attachments.push(CompiledAttachment {
                image: usage.image,
                level,
                depth,
                layout: usage
                    .usage
                    .access(Access::sampled_layout(usage_flags[usage.image]))
                    .layout,
                load_op: attachment.load_op(),
                store_op: store_ops[index],
                clear_value: attachment.clear_value(),
            });
        }

        Ok(())
    }

    fn error(&self, position: usize, reason: String) -> RenderGraphError {
        RenderGraphError::Declaration {
            pass: self.declaration.passes[self.live_passes[position]].name,
            reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use ash::vk;

    use super::GraphCompiler;
    use crate::barrier_source::BarrierSource;
    use crate::compiled_graph::CompiledGraph;
    use crate::declared_buffer::DeclaredBuffer;
    use crate::declared_image::DeclaredImage;
    use crate::frame_declaration::FrameDeclaration;
    use crate::{
        Attachment, BufferHandle, BufferState, BufferUsage, GraphImageDescription, HistoryId, ImageHandle, ImageSize,
        ImageState, ImportedImageHandle, PassDeclaration, RenderGraphError, Stage,
    };

    const COLOR_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;
    const BLOOM_LEVELS: u32 = 3;
    const SWAPCHAIN_ACQUIRED: ImageState = ImageState {
        stages: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
        access: vk::AccessFlags2::NONE,
        layout: vk::ImageLayout::UNDEFINED,
    };
    const PRESENTED: ImageState = ImageState {
        stages: vk::PipelineStageFlags2::NONE,
        access: vk::AccessFlags2::NONE,
        layout: vk::ImageLayout::PRESENT_SRC_KHR,
    };
    const HOST_READ: BufferState = BufferState {
        stages: vk::PipelineStageFlags2::HOST,
        access: vk::AccessFlags2::HOST_READ,
    };
    const UNUSED_BUFFER: BufferState = BufferState {
        stages: vk::PipelineStageFlags2::NONE,
        access: vk::AccessFlags2::NONE,
    };
    const COLOR_WRITTEN: ImageState = ImageState {
        stages: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
        access: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    };

    fn color(name: &'static str, mip_levels: u32) -> GraphImageDescription {
        GraphImageDescription {
            name,
            format: COLOR_FORMAT,
            size: ImageSize::Output,
            mip_levels,
        }
    }

    fn import_swapchain(frame: &mut FrameDeclaration) -> ImportedImageHandle {
        frame.add_imported_image(DeclaredImage::Imported {
            import: 0,
            name: "swapchain",
            aspect: vk::ImageAspectFlags::COLOR,
            initial: SWAPCHAIN_ACQUIRED,
            final_state: PRESENTED,
        })
    }

    fn import_readback(frame: &mut FrameDeclaration) -> BufferHandle {
        frame.add_buffer(DeclaredBuffer {
            name: "readback",
            initial: UNUSED_BUFFER,
            final_state: HOST_READ,
        })
    }

    fn compile(frame: &FrameDeclaration) -> Result<CompiledGraph, RenderGraphError> {
        GraphCompiler::new(frame).compile()
    }

    fn declaration_reason(result: Result<CompiledGraph, RenderGraphError>) -> String {
        match result {
            Err(RenderGraphError::Declaration { reason, .. }) => reason,
            Err(other) => panic!("expected a declaration error, got {other}"),
            Ok(_) => panic!("expected a declaration error, the graph compiled"),
        }
    }

    // Forward writes scene color; tonemap samples it into the swapchain.
    fn forward_and_tonemap(frame: &mut FrameDeclaration) -> ImageHandle {
        let swapchain = import_swapchain(frame);
        let mut forward = PassDeclaration::new(frame, &[], "forward");
        let scene = forward.create_image(color("scene", 1));
        forward.color_attachment(scene, Attachment::ClearColor([0.0; 4]));
        let mut tonemap = PassDeclaration::new(frame, &[], "tonemap");
        tonemap.sampled(scene, Stage::Fragment);
        tonemap.color_attachment(swapchain, Attachment::DontCare);

        scene
    }

    #[test]
    fn a_written_image_is_transitioned_for_its_reader() {
        let mut frame = FrameDeclaration::default();
        let scene = forward_and_tonemap(&mut frame);
        let compiled = compile(&frame).unwrap();

        let forward = &compiled.passes[0].steps[0].image_barriers;
        assert_eq!(forward.len(), 1);
        assert_eq!(forward[0].source, BarrierSource::PreviousFrame { discard: true });
        assert_eq!(forward[0].new_layout, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let tonemap = &compiled.passes[1].steps[0].image_barriers;
        let read = tonemap.iter().find(|barrier| barrier.image == scene.index).unwrap();
        assert_eq!(read.source, BarrierSource::Known(COLOR_WRITTEN));
        assert_eq!(read.destination_stages, vk::PipelineStageFlags2::FRAGMENT_SHADER);
        assert_eq!(read.destination_access, vk::AccessFlags2::SHADER_SAMPLED_READ);
        assert_eq!(read.new_layout, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
    }

    #[test]
    fn the_swapchain_waits_on_acquire_and_ends_presentable() {
        let mut frame = FrameDeclaration::default();
        forward_and_tonemap(&mut frame);
        let compiled = compile(&frame).unwrap();

        let first = compiled.passes[1].steps[0]
            .image_barriers
            .iter()
            .find(|barrier| barrier.image == 0)
            .unwrap();
        assert_eq!(first.source, BarrierSource::Known(SWAPCHAIN_ACQUIRED));

        assert_eq!(compiled.final_image_barriers.len(), 1);
        let present = compiled.final_image_barriers[0];
        assert_eq!(present.source, BarrierSource::Known(COLOR_WRITTEN));
        assert_eq!(present.new_layout, vk::ImageLayout::PRESENT_SRC_KHR);
    }

    #[test]
    fn reads_in_several_stages_share_one_barrier() {
        let mut frame = FrameDeclaration::default();
        let scene = forward_and_tonemap(&mut frame);
        let readback = import_readback(&mut frame);
        let mut histogram = PassDeclaration::new(&mut frame, &[], "histogram");
        histogram.sampled(scene, Stage::Compute);
        histogram.buffer(readback, BufferUsage::StorageReadWrite(Stage::Compute));
        let compiled = compile(&frame).unwrap();

        let reads: Vec<_> = compiled
            .passes
            .iter()
            .flat_map(|pass| &pass.steps)
            .flat_map(|step| &step.image_barriers)
            .filter(|barrier| {
                barrier.image == scene.index && barrier.new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
            })
            .collect();
        assert_eq!(reads.len(), 1);
        assert_eq!(
            reads[0].destination_stages,
            vk::PipelineStageFlags2::FRAGMENT_SHADER | vk::PipelineStageFlags2::COMPUTE_SHADER
        );
    }

    #[test]
    fn an_attachment_nothing_reads_later_is_not_stored() {
        let mut frame = FrameDeclaration::default();
        let swapchain = import_swapchain(&mut frame);
        let mut forward = PassDeclaration::new(&mut frame, &[], "forward");
        let scene = forward.create_image(color("scene", 1));
        let motion = forward.create_image(color("motion", 1));
        forward.color_attachment(scene, Attachment::ClearColor([0.0; 4]));
        forward.color_attachment(motion, Attachment::ClearColor([0.0; 4]));
        let mut tonemap = PassDeclaration::new(&mut frame, &[], "tonemap");
        tonemap.sampled(scene, Stage::Fragment);
        tonemap.color_attachment(swapchain, Attachment::DontCare);
        let compiled = compile(&frame).unwrap();

        let attachments = &compiled.passes[0].steps[0].attachments;
        assert_eq!(attachments[0].store_op, vk::AttachmentStoreOp::STORE);
        assert_eq!(attachments[1].store_op, vk::AttachmentStoreOp::DONT_CARE);
        assert_eq!(attachments[1].load_op, vk::AttachmentLoadOp::CLEAR);
        assert_eq!(
            compiled.passes[1].steps[0].attachments[0].store_op,
            vk::AttachmentStoreOp::STORE
        );
    }

    #[test]
    fn a_pass_nothing_reads_is_culled_unless_kept() {
        let mut frame = FrameDeclaration::default();
        forward_and_tonemap(&mut frame);
        let mut unused = PassDeclaration::new(&mut frame, &[], "unused");
        let image = unused.create_image(color("unused", 1));
        unused.color_attachment(image, Attachment::DontCare);
        let mut kept = PassDeclaration::new(&mut frame, &[], "kept");
        let image = kept.create_image(color("kept", 1));
        kept.color_attachment(image, Attachment::DontCare);
        kept.keep();
        let compiled = compile(&frame).unwrap();

        let names: Vec<_> = compiled
            .passes
            .iter()
            .map(|pass| frame.passes[pass.declaration].name)
            .collect();
        assert_eq!(names, ["forward", "tonemap", "kept"]);
    }

    #[test]
    fn a_producer_whose_only_reader_is_culled_is_culled() {
        let mut frame = FrameDeclaration::default();
        forward_and_tonemap(&mut frame);
        let mut producer = PassDeclaration::new(&mut frame, &[], "producer");
        let image = producer.create_image(color("intermediate", 1));
        producer.color_attachment(image, Attachment::DontCare);
        let mut consumer = PassDeclaration::new(&mut frame, &[], "consumer");
        consumer.sampled(image, Stage::Fragment);
        let output = consumer.create_image(color("unread", 1));
        consumer.color_attachment(output, Attachment::DontCare);
        let compiled = compile(&frame).unwrap();

        assert_eq!(compiled.passes.len(), 2);
        assert!(compiled.usage_flags[image.index].is_empty());
    }

    #[test]
    fn a_pass_writing_an_imported_buffer_is_kept() {
        let mut frame = FrameDeclaration::default();
        let readback = import_readback(&mut frame);
        let mut clear = PassDeclaration::new(&mut frame, &[], "clear");
        clear.buffer(readback, BufferUsage::TransferDestination);
        clear.next_step();
        clear.buffer(readback, BufferUsage::StorageReadWrite(Stage::Compute));
        let compiled = compile(&frame).unwrap();

        let steps = &compiled.passes[0].steps;
        assert_eq!(steps[0].buffer_barriers[0].source_stages, vk::PipelineStageFlags2::NONE);
        let between = steps[1].buffer_barriers[0];
        assert_eq!(between.source_access, vk::AccessFlags2::TRANSFER_WRITE);
        assert_eq!(between.destination_stages, vk::PipelineStageFlags2::COMPUTE_SHADER);

        let host = compiled.final_buffer_barriers[0];
        assert_eq!(host.source_access, vk::AccessFlags2::SHADER_STORAGE_WRITE);
        assert_eq!(host.destination_stages, vk::PipelineStageFlags2::HOST);
        assert_eq!(host.destination_access, vk::AccessFlags2::HOST_READ);
    }

    #[test]
    fn reading_an_image_before_any_write_is_rejected() {
        let mut frame = FrameDeclaration::default();
        let swapchain = import_swapchain(&mut frame);
        let mut tonemap = PassDeclaration::new(&mut frame, &[], "tonemap");
        let scene = tonemap.create_image(color("scene", 1));
        tonemap.sampled(scene, Stage::Fragment);
        tonemap.color_attachment(swapchain, Attachment::DontCare);

        assert_eq!(
            declaration_reason(compile(&frame)),
            "reads scene before any pass writes it"
        );
    }

    #[test]
    fn loading_an_attachment_counts_as_a_read() {
        let mut frame = FrameDeclaration::default();
        let mut overlay = PassDeclaration::new(&mut frame, &[], "overlay");
        let image = overlay.create_image(color("overlay", 1));
        overlay.color_attachment(image, Attachment::Load);
        overlay.keep();

        assert_eq!(
            declaration_reason(compile(&frame)),
            "reads overlay before any pass writes it"
        );
    }

    #[test]
    fn conflicting_usages_in_one_step_are_rejected() {
        let mut frame = FrameDeclaration::default();
        let mut pass = PassDeclaration::new(&mut frame, &[], "feedback");
        let image = pass.create_image(color("feedback", 1));
        pass.storage_write(image, Stage::Compute);
        pass.sampled(image, Stage::Compute);
        pass.keep();

        assert_eq!(
            declaration_reason(compile(&frame)),
            "uses feedback twice in one step with conflicting usages"
        );
    }

    #[test]
    fn an_attachment_on_a_mipmapped_image_must_name_a_level() {
        let mut frame = FrameDeclaration::default();
        let mut pass = PassDeclaration::new(&mut frame, &[], "mips");
        let image = pass.create_image(color("chain", BLOOM_LEVELS));
        pass.color_attachment(image, Attachment::DontCare);
        pass.keep();

        assert_eq!(
            declaration_reason(compile(&frame)),
            "renders to chain, which has several levels, without naming one"
        );
    }

    #[test]
    fn a_level_outside_the_image_is_rejected() {
        let mut frame = FrameDeclaration::default();
        let mut pass = PassDeclaration::new(&mut frame, &[], "mips");
        let image = pass.create_image(color("chain", BLOOM_LEVELS));
        pass.storage_write(image.level(BLOOM_LEVELS), Stage::Compute);
        pass.keep();

        assert_eq!(
            declaration_reason(compile(&frame)),
            "uses level 3 of chain, which has 3"
        );
    }

    // Each level is downsampled from the one before it, then the chain is sampled whole: the barrier before each
    // level's first read already covers the composite's later read.
    #[test]
    fn steps_on_mip_levels_get_barriers_between_them() {
        let mut frame = FrameDeclaration::default();
        let scene = forward_and_tonemap(&mut frame);
        let mut bloom = PassDeclaration::new(&mut frame, &[], "bloom");
        let chain = bloom.create_image(color("bloom", BLOOM_LEVELS));
        bloom.sampled(scene, Stage::Compute);
        bloom.storage_write(chain.level(0), Stage::Compute);

        for level in 1..BLOOM_LEVELS {
            bloom.next_step();
            bloom.sampled(chain.level(level - 1), Stage::Compute);
            bloom.storage_write(chain.level(level), Stage::Compute);
        }

        let mut composite = PassDeclaration::new(&mut frame, &[], "composite");
        composite.sampled(chain, Stage::Fragment);
        let output = composite.create_image(color("composite", 1));
        composite.color_attachment(output, Attachment::DontCare);
        composite.keep();
        let compiled = compile(&frame).unwrap();

        let bloom = &compiled.passes[2];
        assert_eq!(bloom.steps.len(), BLOOM_LEVELS as usize);

        for level in 1..BLOOM_LEVELS {
            let barriers = &bloom.steps[level as usize].image_barriers;
            let read = barriers.iter().find(|barrier| barrier.base_level == level - 1).unwrap();
            assert_eq!(
                read.source,
                BarrierSource::Known(ImageState {
                    stages: vk::PipelineStageFlags2::COMPUTE_SHADER,
                    access: vk::AccessFlags2::SHADER_STORAGE_WRITE,
                    layout: vk::ImageLayout::GENERAL,
                })
            );
            assert_eq!(read.destination_access, vk::AccessFlags2::SHADER_SAMPLED_READ);
            assert!(barriers.iter().any(|barrier| barrier.base_level == level
                && barrier.source == BarrierSource::PreviousFrame { discard: true }));
        }

        let whole = &compiled.passes[3].steps[0]
            .image_barriers
            .iter()
            .filter(|barrier| barrier.image == chain.index)
            .collect::<Vec<_>>();
        assert_eq!(whole.len(), 1);
        assert_eq!(whole[0].base_level, BLOOM_LEVELS - 1);

        let first_read = bloom.steps[1]
            .image_barriers
            .iter()
            .find(|barrier| barrier.base_level == 0)
            .unwrap();
        assert_eq!(
            first_read.destination_stages,
            vk::PipelineStageFlags2::COMPUTE_SHADER | vk::PipelineStageFlags2::FRAGMENT_SHADER
        );
    }

    #[test]
    fn storage_images_are_sampled_in_the_general_layout() {
        let mut frame = FrameDeclaration::default();
        let mut pass = PassDeclaration::new(&mut frame, &[], "compute");
        let image = pass.create_image(color("storage", 1));
        pass.storage_write(image, Stage::Compute);
        pass.next_step();
        pass.sampled(image, Stage::Compute);
        pass.keep();
        let compiled = compile(&frame).unwrap();

        let read = compiled.passes[0].steps[1].image_barriers[0];
        assert_eq!(read.new_layout, vk::ImageLayout::GENERAL);
        assert_eq!(
            compiled.usage_flags[image.index],
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED
        );
    }

    #[test]
    fn history_images_keep_their_contents_and_share_usage_flags() {
        let histories = [color("history", 1)];
        let id = HistoryId { index: 0 };
        let mut frame = FrameDeclaration::default();
        let swapchain = import_swapchain(&mut frame);
        let mut taa = PassDeclaration::new(&mut frame, &histories, "taa");
        let history = taa.history(id);
        taa.sampled(history.previous, Stage::Compute);
        taa.storage_write(history.current, Stage::Compute);
        let mut tonemap = PassDeclaration::new(&mut frame, &histories, "tonemap");
        tonemap.sampled(history.current, Stage::Fragment);
        tonemap.color_attachment(swapchain, Attachment::DontCare);
        let compiled = compile(&frame).unwrap();

        let taa = &compiled.passes[0].steps[0].image_barriers;
        let previous = taa
            .iter()
            .find(|barrier| barrier.image == history.previous.index)
            .unwrap();
        let current = taa
            .iter()
            .find(|barrier| barrier.image == history.current.index)
            .unwrap();
        assert_eq!(previous.source, BarrierSource::PreviousFrame { discard: false });
        assert_eq!(current.source, BarrierSource::PreviousFrame { discard: true });
        assert_eq!(previous.new_layout, vk::ImageLayout::GENERAL);

        let flags = vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED;
        assert_eq!(compiled.usage_flags[history.previous.index], flags);
        assert_eq!(compiled.usage_flags[history.current.index], flags);
        assert!(
            compiled.end_states.iter().any(|end| end.image == history.current.index
                && end.state.stages == vk::PipelineStageFlags2::FRAGMENT_SHADER)
        );
    }

    #[test]
    fn a_history_is_one_image_pair_however_often_it_is_named() {
        let histories = [color("history", 1)];
        let id = HistoryId { index: 0 };
        let mut frame = FrameDeclaration::default();
        let first = PassDeclaration::new(&mut frame, &histories, "first").history(id);
        let second = PassDeclaration::new(&mut frame, &histories, "second").history(id);

        assert_eq!(first, second);
        assert_eq!(frame.images.len(), 2);
    }

    #[test]
    fn equal_declarations_compare_equal() {
        let mut first = FrameDeclaration::default();
        forward_and_tonemap(&mut first);
        let mut second = FrameDeclaration::default();
        forward_and_tonemap(&mut second);
        assert_eq!(first, second);

        let mut bloom = PassDeclaration::new(&mut second, &[], "bloom");
        bloom.keep();
        assert_ne!(first, second);
    }
}
