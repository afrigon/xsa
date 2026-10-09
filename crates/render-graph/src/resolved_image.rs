use ash::vk;

use crate::physical_image::PhysicalImage;
use crate::{ImageId, ImportedImage};

// What an image of this frame is: a pooled image, an imported one, or nothing for an image no running pass uses.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ResolvedImage {
    pub name: &'static str,
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub extent: vk::Extent2D,
    pub aspect: vk::ImageAspectFlags,
    pub id: Option<ImageId>,
}

impl ResolvedImage {
    pub fn unused(name: &'static str) -> ResolvedImage {
        ResolvedImage {
            name,
            image: vk::Image::null(),
            view: vk::ImageView::null(),
            extent: vk::Extent2D { width: 0, height: 0 },
            aspect: vk::ImageAspectFlags::empty(),
            id: None,
        }
    }

    pub fn imported(imported: &ImportedImage) -> ResolvedImage {
        ResolvedImage {
            name: imported.name,
            image: imported.image,
            view: imported.view,
            extent: imported.extent,
            aspect: imported.aspect,
            id: None,
        }
    }

    pub fn physical(id: ImageId, physical: &PhysicalImage) -> ResolvedImage {
        ResolvedImage {
            name: physical.description.name,
            image: physical.image,
            view: physical.view,
            extent: physical.extent,
            aspect: physical.description.aspect(),
            id: Some(id),
        }
    }

    pub fn level_extent(&self, level: u32) -> vk::Extent2D {
        vk::Extent2D {
            width: (self.extent.width >> level).max(1),
            height: (self.extent.height >> level).max(1),
        }
    }
}
