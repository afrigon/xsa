use crate::declared_buffer_usage::DeclaredBufferUsage;
use crate::declared_image::DeclaredImage;
use crate::declared_image_usage::DeclaredImageUsage;
use crate::declared_pass::DeclaredPass;
use crate::frame_declaration::FrameDeclaration;
use crate::{
    Attachment, BufferHandle, BufferUsage, GraphImageDescription, HistoryId, HistoryImage, ImageHandle, ResourceUsage,
    Stage, Subresource,
};

/// What a pass creates and uses this frame. Usages apply to the current step; `next_step` starts another.
pub struct PassDeclaration<'frame> {
    frame: &'frame mut FrameDeclaration,
    histories: &'frame [GraphImageDescription],
    pass: usize,
    step: u32,
}

impl<'frame> PassDeclaration<'frame> {
    pub(crate) fn new(
        frame: &'frame mut FrameDeclaration,
        histories: &'frame [GraphImageDescription],
        name: &'static str,
    ) -> PassDeclaration<'frame> {
        frame.passes.push(DeclaredPass {
            name,
            step_count: 1,
            keep: false,
        });
        let pass = frame.passes.len() - 1;

        PassDeclaration {
            frame,
            histories,
            pass,
            step: 0,
        }
    }

    pub fn create_image(&mut self, description: GraphImageDescription) -> ImageHandle {
        self.frame.add_image(DeclaredImage::Transient(description))
    }

    pub fn history(&mut self, id: HistoryId) -> HistoryImage {
        let description = self.histories[id.index];

        HistoryImage {
            current: self.frame.history(id, description, false),
            previous: self.frame.history(id, description, true),
        }
    }

    /// Starts a step that runs after a barrier on everything the previous steps wrote.
    pub fn next_step(&mut self) {
        self.step += 1;
        self.frame.passes[self.pass].step_count += 1;
    }

    /// Keeps the pass even when nothing the graph can see reads what it writes.
    pub fn keep(&mut self) {
        self.frame.passes[self.pass].keep = true;
    }

    pub fn image(&mut self, subresource: impl Into<Subresource>, usage: ResourceUsage) {
        let subresource = subresource.into();
        self.frame.image_usages.push(DeclaredImageUsage {
            pass: self.pass,
            step: self.step,
            image: subresource.image.index,
            level: subresource.level,
            usage,
        });
    }

    pub fn buffer(&mut self, buffer: BufferHandle, usage: BufferUsage) {
        self.frame.buffer_usages.push(DeclaredBufferUsage {
            pass: self.pass,
            step: self.step,
            buffer: buffer.index,
            usage,
        });
    }

    pub fn color_attachment(&mut self, target: impl Into<Subresource>, attachment: Attachment) {
        self.image(target, ResourceUsage::ColorAttachment(attachment));
    }

    pub fn depth_attachment(&mut self, target: impl Into<Subresource>, attachment: Attachment) {
        self.image(target, ResourceUsage::DepthAttachment(attachment));
    }

    pub fn sampled(&mut self, source: impl Into<Subresource>, stage: Stage) {
        self.image(source, ResourceUsage::Sampled(stage));
    }

    pub fn storage_read(&mut self, source: impl Into<Subresource>, stage: Stage) {
        self.image(source, ResourceUsage::StorageRead(stage));
    }

    pub fn storage_write(&mut self, target: impl Into<Subresource>, stage: Stage) {
        self.image(target, ResourceUsage::StorageWrite(stage));
    }

    pub fn storage_read_write(&mut self, target: impl Into<Subresource>, stage: Stage) {
        self.image(target, ResourceUsage::StorageReadWrite(stage));
    }

    pub fn transfer_source(&mut self, source: impl Into<Subresource>) {
        self.image(source, ResourceUsage::TransferSource);
    }

    pub fn transfer_destination(&mut self, target: impl Into<Subresource>) {
        self.image(target, ResourceUsage::TransferDestination);
    }
}
