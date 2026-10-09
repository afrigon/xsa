use crate::declared_buffer::DeclaredBuffer;
use crate::declared_buffer_usage::DeclaredBufferUsage;
use crate::declared_image::DeclaredImage;
use crate::declared_image_usage::DeclaredImageUsage;
use crate::declared_pass::DeclaredPass;
use crate::{BufferHandle, GraphImageDescription, HistoryId, ImageHandle};

// Everything a frame declared, in declaration order. Two frames with equal declarations compile to the same graph.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FrameDeclaration {
    pub passes: Vec<DeclaredPass>,
    pub images: Vec<DeclaredImage>,
    pub buffers: Vec<DeclaredBuffer>,
    pub image_usages: Vec<DeclaredImageUsage>,
    pub buffer_usages: Vec<DeclaredBufferUsage>,
}

impl FrameDeclaration {
    pub fn clear(&mut self) {
        self.passes.clear();
        self.images.clear();
        self.buffers.clear();
        self.image_usages.clear();
        self.buffer_usages.clear();
    }

    pub fn add_image(&mut self, image: DeclaredImage) -> ImageHandle {
        self.images.push(image);

        ImageHandle {
            index: self.images.len() - 1,
        }
    }

    pub fn add_buffer(&mut self, buffer: DeclaredBuffer) -> BufferHandle {
        self.buffers.push(buffer);

        BufferHandle {
            index: self.buffers.len() - 1,
        }
    }

    pub fn history(&mut self, id: HistoryId, description: GraphImageDescription, previous: bool) -> ImageHandle {
        let image = DeclaredImage::History {
            id,
            description,
            previous,
        };

        match self.images.iter().position(|declared| *declared == image) {
            Some(index) => ImageHandle { index },
            None => self.add_image(image),
        }
    }
}
