use crate::declared_buffer::DeclaredBuffer;
use crate::declared_buffer_usage::DeclaredBufferUsage;
use crate::declared_image::DeclaredImage;
use crate::declared_image_usage::DeclaredImageUsage;
use crate::declared_pass::DeclaredPass;
use crate::{BufferHandle, GraphImageDescription, HistoryId, HistoryImage, ImageHandle, ImportedImageHandle};

// Everything a frame declared, in declaration order. Two frames with equal declarations compile to the same graph.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FrameDeclaration {
    pub passes: Vec<DeclaredPass>,
    pub images: Vec<DeclaredImage>,
    pub buffers: Vec<DeclaredBuffer>,
    pub image_usages: Vec<DeclaredImageUsage>,
    pub buffer_usages: Vec<DeclaredBufferUsage>,
    pub histories: Vec<Option<HistoryImage>>,
}

impl FrameDeclaration {
    pub fn clear(&mut self) {
        self.passes.clear();
        self.images.clear();
        self.buffers.clear();
        self.image_usages.clear();
        self.buffer_usages.clear();
        self.histories.clear();
    }

    pub fn add_image(&mut self, image: DeclaredImage) -> ImageHandle {
        ImageHandle {
            index: self.push_image(image),
        }
    }

    pub fn add_imported_image(&mut self, image: DeclaredImage) -> ImportedImageHandle {
        ImportedImageHandle {
            index: self.push_image(image),
        }
    }

    pub fn add_buffer(&mut self, buffer: DeclaredBuffer) -> BufferHandle {
        self.buffers.push(buffer);

        BufferHandle {
            index: self.buffers.len() - 1,
        }
    }

    // A history named by several passes is one pair of images in the frame.
    pub fn history(&mut self, id: HistoryId, description: GraphImageDescription) -> HistoryImage {
        if let Some(Some(history)) = self.histories.get(id.index) {
            return *history;
        }

        let history = HistoryImage {
            current: self.add_image(DeclaredImage::History {
                id,
                description,
                previous: false,
            }),
            previous: self.add_image(DeclaredImage::History {
                id,
                description,
                previous: true,
            }),
        };

        if self.histories.len() <= id.index {
            self.histories.resize(id.index + 1, None);
        }

        self.histories[id.index] = Some(history);

        history
    }

    fn push_image(&mut self, image: DeclaredImage) -> usize {
        self.images.push(image);

        self.images.len() - 1
    }
}
