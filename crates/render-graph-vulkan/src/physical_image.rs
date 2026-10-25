use ash::vk;
use memory_vulkan::{DeviceObject, Image, ImageDescription, ImageKind, ImageMemory, MemoryContext, QueueSharing};

use crate::access::Access;
use crate::image_owner::ImageOwner;
use crate::{GraphImageDescription, ImageState, RenderGraphError};

pub(crate) struct PhysicalImage {
    // Declaration order is destruction order: the level views before the image they view.
    level_views: Vec<DeviceObject<vk::ImageView>>,
    image: Image,
    pub extent: vk::Extent2D,
    pub description: GraphImageDescription,
    pub usage: vk::ImageUsageFlags,
    pub owner: ImageOwner,
    pub states: Vec<ImageState>,
}

impl PhysicalImage {
    pub fn new(
        memory: &MemoryContext,
        description: GraphImageDescription,
        usage: vk::ImageUsageFlags,
        output: vk::Extent2D,
        owner: ImageOwner,
    ) -> Result<PhysicalImage, RenderGraphError> {
        let extent = description.size.extent(output);
        let aspect = description.aspect();
        let view_aspect = if aspect.contains(vk::ImageAspectFlags::DEPTH) {
            vk::ImageAspectFlags::DEPTH
        } else {
            aspect
        };
        let image = Image::new(
            memory,
            &ImageDescription {
                name: description.name,
                extent,
                format: description.format,
                usage,
                aspect: view_aspect,
                mip_levels: description.mip_levels,
                kind: ImageKind::Single,
                memory: ImageMemory::Shared,
                sharing: QueueSharing::Exclusive,
            },
        )?;
        let level_views = if description.mip_levels > 1 {
            (0..description.mip_levels)
                .map(|level| image.create_view(memory, level, 1))
                .collect::<Result<_, _>>()?
        } else {
            Vec::new()
        };

        Ok(PhysicalImage {
            level_views,
            image,
            extent,
            description,
            usage,
            owner,
            states: vec![ImageState::UNUSED; description.mip_levels as usize],
        })
    }

    pub fn image(&self) -> vk::Image {
        self.image.handle()
    }

    pub fn view(&self) -> vk::ImageView {
        self.image.view()
    }

    pub fn level_view(&self, level: u32) -> vk::ImageView {
        match self.level_views.get(level as usize) {
            Some(view) => view.handle(),
            None => self.image.view(),
        }
    }

    pub fn sampled_layout(&self) -> vk::ImageLayout {
        Access::sampled_layout(self.usage)
    }
}
