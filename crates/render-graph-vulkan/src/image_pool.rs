use std::collections::HashMap;

use ash::vk;
use memory_vulkan::MemoryContext;

use crate::declared_image::DeclaredImage;
use crate::frame_declaration::FrameDeclaration;
use crate::image_owner::ImageOwner;
use crate::image_transient_key::ImageTransientKey;
use crate::physical_image::PhysicalImage;
use crate::{GraphImageDescription, HistoryId, ImageId, ImageSize, RenderGraphError};

pub(crate) const HISTORY_IMAGE_COUNT: usize = 2;

// The images the graph allocates. An image is never freed while the graph lives, so its ImageId stays valid.
pub(crate) struct ImagePool {
    output: vk::Extent2D,
    images: Vec<PhysicalImage>,
    transients: HashMap<ImageTransientKey, Vec<ImageId>>,
}

impl ImagePool {
    pub fn new(output: vk::Extent2D) -> ImagePool {
        ImagePool {
            output,
            images: Vec::new(),
            transients: HashMap::new(),
        }
    }

    pub fn image(&self, id: ImageId) -> &PhysicalImage {
        &self.images[id.index]
    }

    pub fn image_mut(&mut self, id: ImageId) -> &mut PhysicalImage {
        &mut self.images[id.index]
    }

    // Images with the same description and usage are reused, each by at most one image of the frame: the frame's
    // nth image of a kind takes the pool's nth image of that kind.
    pub fn assign_transients(
        &mut self,
        memory: &MemoryContext,
        declaration: &FrameDeclaration,
        usage_flags: &[vk::ImageUsageFlags],
        created: &mut Vec<ImageId>,
    ) -> Result<Vec<Option<ImageId>>, RenderGraphError> {
        let mut used: HashMap<ImageTransientKey, usize> = HashMap::new();
        let mut slots = Vec::with_capacity(declaration.images.len());

        for (image, usage) in declaration.images.iter().zip(usage_flags) {
            let DeclaredImage::Transient(description) = image else {
                slots.push(None);
                continue;
            };

            if usage.is_empty() {
                slots.push(None);
                continue;
            }

            let key = ImageTransientKey {
                description: *description,
                usage: *usage,
            };
            let nth = used.entry(key).or_default();
            let existing = self.transients.get(&key).and_then(|ids| ids.get(*nth)).copied();
            let id = match existing {
                Some(id) => id,
                None => {
                    let physical =
                        PhysicalImage::new(memory, *description, *usage, self.output, ImageOwner::Transient)?;
                    let id = self.add(physical);
                    self.transients.entry(key).or_default().push(id);
                    created.push(id);
                    id
                }
            };
            *nth += 1;
            slots.push(Some(id));
        }

        Ok(slots)
    }

    // Returns whether the pair was created. Its usage is fixed by the first frame that uses it.
    pub fn ensure_history(
        &mut self,
        memory: &MemoryContext,
        id: HistoryId,
        description: GraphImageDescription,
        usage: vk::ImageUsageFlags,
        pair: &mut Option<[ImageId; HISTORY_IMAGE_COUNT]>,
    ) -> Result<bool, RenderGraphError> {
        if let Some(ids) = pair {
            if self.images[ids[0].index].usage != usage {
                return Err(RenderGraphError::HistoryUsageChanged {
                    history: description.name,
                });
            }

            return Ok(false);
        }

        let owner = ImageOwner::History { id };
        let first = self.add(PhysicalImage::new(memory, description, usage, self.output, owner)?);
        let second = self.add(PhysicalImage::new(memory, description, usage, self.output, owner)?);
        *pair = Some([first, second]);

        Ok(true)
    }

    // A replaced image is destroyed once the GPU is done with the frames that used it.
    pub fn resize(&mut self, memory: &MemoryContext, output: vk::Extent2D) -> Result<Vec<ImageId>, RenderGraphError> {
        self.output = output;
        let mut recreated = Vec::new();

        for index in 0..self.images.len() {
            let current = &self.images[index];

            if matches!(current.description.size, ImageSize::Fixed(_)) {
                continue;
            }

            let physical = PhysicalImage::new(memory, current.description, current.usage, self.output, current.owner)?;
            self.images[index] = physical;
            recreated.push(ImageId { index });
        }

        Ok(recreated)
    }

    fn add(&mut self, image: PhysicalImage) -> ImageId {
        self.images.push(image);

        ImageId {
            index: self.images.len() - 1,
        }
    }
}
