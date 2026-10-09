use ash::vk;
use gpu_allocator::vulkan::Allocator;

use crate::declared_image::DeclaredImage;
use crate::frame_declaration::FrameDeclaration;
use crate::image_owner::ImageOwner;
use crate::physical_image::PhysicalImage;
use crate::{GraphImageDescription, HistoryId, ImageId, ImageSize, RenderGraphError};

pub(crate) const HISTORY_IMAGE_COUNT: usize = 2;

// The images the graph allocates. An image is never freed while the graph lives, so its ImageId stays valid; one
// replaced while frames may still use it is retired until the GPU is idle.
pub(crate) struct ImagePool {
    output: vk::Extent2D,
    images: Vec<PhysicalImage>,
    retired: Vec<PhysicalImage>,
}

impl ImagePool {
    pub fn new(output: vk::Extent2D) -> ImagePool {
        ImagePool {
            output,
            images: Vec::new(),
            retired: Vec::new(),
        }
    }

    pub fn image(&self, id: ImageId) -> &PhysicalImage {
        &self.images[id.index]
    }

    pub fn image_mut(&mut self, id: ImageId) -> &mut PhysicalImage {
        &mut self.images[id.index]
    }

    // Images with the same description and usage are reused, each by at most one image of the frame.
    pub fn assign_transients(
        &mut self,
        device: &ash::Device,
        allocator: &mut Allocator,
        declaration: &FrameDeclaration,
        usage_flags: &[vk::ImageUsageFlags],
        created: &mut Vec<ImageId>,
    ) -> Result<Vec<Option<ImageId>>, RenderGraphError> {
        let mut taken = vec![false; self.images.len()];
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

            let reusable = self.images.iter().zip(&taken).position(|(physical, taken)| {
                !taken
                    && physical.owner == ImageOwner::Transient
                    && physical.description == *description
                    && physical.usage == *usage
            });
            let id = match reusable {
                Some(index) => ImageId { index },
                None => {
                    let physical = PhysicalImage::new(
                        device,
                        allocator,
                        *description,
                        *usage,
                        self.output,
                        ImageOwner::Transient,
                    )?;
                    let id = self.add(physical);
                    taken.push(false);
                    created.push(id);
                    id
                }
            };
            taken[id.index] = true;
            slots.push(Some(id));
        }

        Ok(slots)
    }

    // Returns whether the pair was created or replaced, which happens when its usage changes.
    pub fn ensure_history(
        &mut self,
        device: &ash::Device,
        allocator: &mut Allocator,
        id: HistoryId,
        description: GraphImageDescription,
        usage: vk::ImageUsageFlags,
        pair: &mut Option<[ImageId; HISTORY_IMAGE_COUNT]>,
    ) -> Result<bool, RenderGraphError> {
        let owner = ImageOwner::History { id };

        match pair {
            Some(ids) if self.images[ids[0].index].usage == usage => Ok(false),
            Some(ids) => {
                for image in ids.iter() {
                    let physical = PhysicalImage::new(device, allocator, description, usage, self.output, owner)?;
                    let previous = std::mem::replace(&mut self.images[image.index], physical);
                    self.retired.push(previous);
                }

                Ok(true)
            }
            None => {
                let first = self.add(PhysicalImage::new(
                    device,
                    allocator,
                    description,
                    usage,
                    self.output,
                    owner,
                )?);
                let second = self.add(PhysicalImage::new(
                    device,
                    allocator,
                    description,
                    usage,
                    self.output,
                    owner,
                )?);
                *pair = Some([first, second]);

                Ok(true)
            }
        }
    }

    // Safety: the GPU must be done with every image.
    pub unsafe fn resize(
        &mut self,
        device: &ash::Device,
        allocator: &mut Allocator,
        output: vk::Extent2D,
    ) -> Result<Vec<ImageId>, RenderGraphError> {
        self.output = output;
        unsafe { self.destroy_retired(device, allocator) }?;
        let mut recreated = Vec::new();

        for index in 0..self.images.len() {
            let current = &self.images[index];

            if matches!(current.description.size, ImageSize::Fixed(_)) {
                continue;
            }

            let physical = PhysicalImage::new(
                device,
                allocator,
                current.description,
                current.usage,
                self.output,
                current.owner,
            )?;
            let mut previous = std::mem::replace(&mut self.images[index], physical);
            unsafe { previous.destroy(device, allocator) }?;
            recreated.push(ImageId { index });
        }

        Ok(recreated)
    }

    // Safety: the GPU must be done with every image.
    pub unsafe fn destroy(&mut self, device: &ash::Device, allocator: &mut Allocator) -> Result<(), RenderGraphError> {
        unsafe { self.destroy_retired(device, allocator) }?;

        for mut image in self.images.drain(..) {
            unsafe { image.destroy(device, allocator) }?;
        }

        Ok(())
    }

    unsafe fn destroy_retired(
        &mut self,
        device: &ash::Device,
        allocator: &mut Allocator,
    ) -> Result<(), RenderGraphError> {
        for mut image in self.retired.drain(..) {
            unsafe { image.destroy(device, allocator) }?;
        }

        Ok(())
    }

    fn add(&mut self, image: PhysicalImage) -> ImageId {
        self.images.push(image);

        ImageId {
            index: self.images.len() - 1,
        }
    }
}
