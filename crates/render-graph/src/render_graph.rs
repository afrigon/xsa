use std::cell::RefCell;

use ash::vk;
use gpu_allocator::vulkan::Allocator;

use crate::barrier_recorder::BarrierRecorder;
use crate::buffer_pool::BufferPool;
use crate::compiled_entry::CompiledEntry;
use crate::declared_buffer::DeclaredBuffer;
use crate::declared_image::DeclaredImage;
use crate::frame_declaration::FrameDeclaration;
use crate::graph_compiler::GraphCompiler;
use crate::image_pool::{HISTORY_IMAGE_COUNT, ImagePool};
use crate::recorded_pass::RecordedPass;
use crate::resolved_buffer::ResolvedBuffer;
use crate::resolved_image::ResolvedImage;
use crate::{
    GraphBuilder, GraphImageDescription, HistoryId, ImageId, ImportedBuffer, ImportedBufferHandle, ImportedImage,
    ImportedImageHandle, PassContext, PassDeclaration, RenderGraphError, RenderPass,
};

const COMPILED_GRAPH_CAPACITY: usize = 16;

/// Owns the images and buffers passes create, the history images kept across frames and the compiled frames to reuse.
pub struct RenderGraph {
    frame_number: usize,
    frame: FrameDeclaration,
    imported_images: Vec<ImportedImage>,
    imported_buffers: Vec<ImportedBuffer>,
    histories: Vec<GraphImageDescription>,
    history_images: Vec<Option<[ImageId; HISTORY_IMAGE_COUNT]>>,
    pool: ImagePool,
    buffer_pool: BufferPool,
    cache: Vec<CompiledEntry>,
    last_entry: Option<usize>,
    resolved: Vec<ResolvedImage>,
    resolved_buffers: Vec<ResolvedBuffer>,
    image_barriers: RefCell<Vec<vk::ImageMemoryBarrier2<'static>>>,
    buffer_barriers: RefCell<Vec<vk::BufferMemoryBarrier2<'static>>>,
}

impl RenderGraph {
    /// `output` is the size images declared with `ImageSize::Output` are allocated at.
    pub fn new(output: vk::Extent2D) -> RenderGraph {
        RenderGraph {
            frame_number: 0,
            frame: FrameDeclaration::default(),
            imported_images: Vec::new(),
            imported_buffers: Vec::new(),
            histories: Vec::new(),
            history_images: Vec::new(),
            pool: ImagePool::new(output),
            buffer_pool: BufferPool::default(),
            cache: Vec::new(),
            last_entry: None,
            resolved: Vec::new(),
            resolved_buffers: Vec::new(),
            image_barriers: RefCell::new(Vec::new()),
            buffer_barriers: RefCell::new(Vec::new()),
        }
    }

    /// Creates a pair of images kept across frames; passes name it with `PassDeclaration::history`.
    pub fn create_history(&mut self, description: GraphImageDescription) -> HistoryId {
        self.histories.push(description);
        self.history_images.push(None);

        HistoryId {
            index: self.histories.len() - 1,
        }
    }

    pub fn begin_frame<'graph, 'passes, Context>(&'graph mut self) -> GraphBuilder<'graph, 'passes, Context> {
        GraphBuilder::new(self)
    }

    /// Reallocates every image sized from the output and returns their ids; their views changed. History images lose
    /// their contents.
    ///
    /// # Safety
    ///
    /// The GPU must be done with every frame recorded through the graph.
    pub unsafe fn resize(
        &mut self,
        device: &ash::Device,
        allocator: &mut Allocator,
        output: vk::Extent2D,
    ) -> Result<Vec<ImageId>, RenderGraphError> {
        unsafe { self.pool.resize(device, allocator, output) }
    }

    pub fn image_view(&self, id: ImageId) -> vk::ImageView {
        self.pool.image(id).view
    }

    pub fn level_view(&self, id: ImageId, level: u32) -> vk::ImageView {
        self.pool.image(id).level_view(level)
    }

    pub fn mip_levels(&self, id: ImageId) -> u32 {
        self.pool.image(id).description.mip_levels
    }

    pub fn usage(&self, id: ImageId) -> vk::ImageUsageFlags {
        self.pool.image(id).usage
    }

    /// The layout every shader samples the image in, which a bindless descriptor for it must name.
    pub fn sampled_layout(&self, id: ImageId) -> vk::ImageLayout {
        self.pool.image(id).sampled_layout()
    }

    /// # Safety
    ///
    /// The GPU must be done with every frame recorded through the graph.
    pub unsafe fn destroy(&mut self, device: &ash::Device, allocator: &mut Allocator) -> Result<(), RenderGraphError> {
        self.cache.clear();
        self.last_entry = None;
        self.history_images.fill(None);

        unsafe { self.pool.destroy(device, allocator) }?;

        unsafe { self.buffer_pool.destroy(device, allocator) }
    }

    pub(crate) fn begin_declaration(&mut self) {
        self.frame.clear();
        self.imported_images.clear();
        self.imported_buffers.clear();
        self.frame_number = self.frame_number.wrapping_add(1);
    }

    pub(crate) fn import_image(&mut self, image: ImportedImage) -> ImportedImageHandle {
        self.imported_images.push(image);

        self.frame.add_imported_image(DeclaredImage::Imported {
            import: self.imported_images.len() - 1,
            name: image.name,
            aspect: image.aspect,
            initial: image.initial,
            final_state: image.final_state,
        })
    }

    pub(crate) fn import_buffer(&mut self, buffer: ImportedBuffer) -> ImportedBufferHandle {
        self.imported_buffers.push(buffer);

        self.frame.add_imported_buffer(DeclaredBuffer::Imported {
            import: self.imported_buffers.len() - 1,
            name: buffer.name,
            initial: buffer.initial,
            final_state: buffer.final_state,
        })
    }

    pub(crate) fn declare<Context, Pass: RenderPass<Context>>(
        &mut self,
        pass: &Pass,
        inputs: Pass::Inputs,
    ) -> Pass::Resources {
        let mut declaration = PassDeclaration::new(&mut self.frame, &self.histories, Pass::NAME);

        pass.declare(&mut declaration, inputs)
    }

    pub(crate) fn compile(
        &mut self,
        device: &ash::Device,
        allocator: &mut Allocator,
        created: &mut Vec<ImageId>,
    ) -> Result<usize, RenderGraphError> {
        let entry = match self.cached_entry() {
            Some(entry) => entry,
            None => self.compile_entry(device, allocator, created)?,
        };
        self.last_entry = Some(entry);
        self.ensure_histories(device, allocator, entry, created)?;
        self.resolve(entry);

        Ok(entry)
    }

    pub(crate) fn execute<Context>(
        &mut self,
        entry: usize,
        passes: &mut [Box<dyn RecordedPass<Context> + '_>],
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        context: &Context,
    ) -> Result<(), RenderGraphError> {
        let entry = &self.cache[entry];
        let recorder = BarrierRecorder {
            device,
            command_buffer,
            resolved: &self.resolved,
            pool: &self.pool,
            buffers: &self.resolved_buffers,
            buffer_pool: &self.buffer_pool,
            image_barriers: &self.image_barriers,
            buffer_barriers: &self.buffer_barriers,
        };

        for compiled_pass in &entry.compiled.passes {
            let declared = entry.declaration.passes[compiled_pass.declaration];
            let pass_context = PassContext::new(&recorder, compiled_pass);
            passes[compiled_pass.declaration]
                .record(context, &pass_context)
                .map_err(|source| RenderGraphError::Pass {
                    pass: declared.name,
                    source,
                })?;
            let recorded = pass_context.recorded_steps();

            if recorded != declared.step_count {
                return Err(RenderGraphError::StepCount {
                    pass: declared.name,
                    declared: declared.step_count,
                    recorded,
                });
            }
        }

        recorder.record(
            &entry.compiled.final_image_barriers,
            &entry.compiled.final_buffer_barriers,
        );

        for end in &entry.compiled.end_states {
            if let Some(id) = self.resolved[end.image].id {
                self.pool.image_mut(id).states[end.level as usize] = end.state;
            }
        }

        for end in &entry.compiled.buffer_end_states {
            if let Some(id) = self.resolved_buffers[end.buffer].id {
                self.buffer_pool.buffer_mut(id).state = end.state;
            }
        }

        Ok(())
    }

    // Most frames declare what the previous one did, so its entry is compared first.
    fn cached_entry(&self) -> Option<usize> {
        if let Some(last) = self.last_entry
            && self.cache[last].declaration == self.frame
        {
            return Some(last);
        }

        self.cache.iter().position(|entry| entry.declaration == self.frame)
    }

    fn compile_entry(
        &mut self,
        device: &ash::Device,
        allocator: &mut Allocator,
        created: &mut Vec<ImageId>,
    ) -> Result<usize, RenderGraphError> {
        let compiled = GraphCompiler::new(&self.frame).compile()?;
        let image_slots =
            self.pool
                .assign_transients(device, allocator, &self.frame, &compiled.usage_flags, created)?;
        let buffer_slots =
            self.buffer_pool
                .assign_transients(device, allocator, &self.frame, &compiled.buffer_usage_flags)?;

        if self.cache.len() == COMPILED_GRAPH_CAPACITY {
            self.cache.remove(0);
        }

        self.cache.push(CompiledEntry {
            declaration: self.frame.clone(),
            compiled,
            image_slots,
            buffer_slots,
        });

        Ok(self.cache.len() - 1)
    }

    fn ensure_histories(
        &mut self,
        device: &ash::Device,
        allocator: &mut Allocator,
        entry: usize,
        created: &mut Vec<ImageId>,
    ) -> Result<(), RenderGraphError> {
        let entry = &self.cache[entry];

        for (image, usage) in entry.declaration.images.iter().zip(&entry.compiled.usage_flags) {
            if let DeclaredImage::History { id, description, .. } = image
                && !usage.is_empty()
            {
                let pair = &mut self.history_images[id.index];

                if self
                    .pool
                    .ensure_history(device, allocator, *id, *description, *usage, pair)?
                    && let Some(ids) = pair
                {
                    created.extend(ids.iter());
                }
            }
        }

        Ok(())
    }

    // Images and buffers a pass declared this frame become the pooled or imported ones recorded with; history images
    // swap roles every frame.
    fn resolve(&mut self, entry: usize) {
        let entry = &self.cache[entry];
        self.resolved.clear();

        for (index, image) in entry.declaration.images.iter().enumerate() {
            let id = match image {
                DeclaredImage::Transient(_) => entry.image_slots[index],
                DeclaredImage::History { id, previous, .. } => self.history_images[id.index]
                    .map(|pair| pair[(self.frame_number + usize::from(*previous)) % HISTORY_IMAGE_COUNT]),
                DeclaredImage::Imported { import, .. } => {
                    self.resolved
                        .push(ResolvedImage::imported(&self.imported_images[*import]));
                    continue;
                }
            };
            let resolved = id.map_or(ResolvedImage::unused(image.name()), |id| {
                ResolvedImage::physical(id, self.pool.image(id))
            });
            self.resolved.push(resolved);
        }

        self.resolved_buffers.clear();

        for (index, buffer) in entry.declaration.buffers.iter().enumerate() {
            let resolved = match buffer {
                DeclaredBuffer::Transient(_) => entry.buffer_slots[index]
                    .map_or(ResolvedBuffer::unused(buffer.name()), |id| {
                        ResolvedBuffer::physical(id, self.buffer_pool.buffer(id))
                    }),
                DeclaredBuffer::Imported { import, .. } => ResolvedBuffer::imported(&self.imported_buffers[*import]),
            };
            self.resolved_buffers.push(resolved);
        }
    }
}
