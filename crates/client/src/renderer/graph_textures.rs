use ash::vk;
use memory_vulkan::{BindlessBinding, BindlessSlot, BindlessTable};
use render_graph_vulkan::{ImageId, RenderGraph};

#[derive(Default)]
struct GraphTexture {
    texture: Option<BindlessSlot>,
    storage_images: Vec<BindlessSlot>,
}

// The bindless slots of the images the render graph allocates, by image id. A recreated image keeps its id and gets
// fresh slots, since frames in flight may still read the old ones.
#[derive(Default)]
pub(super) struct GraphTextures {
    images: Vec<GraphTexture>,
}

impl GraphTextures {
    pub fn register(&mut self, bindless: &BindlessTable, graph: &RenderGraph, ids: &[ImageId]) -> anyhow::Result<()> {
        for id in ids {
            if self.images.len() <= id.index() {
                self.images.resize_with(id.index() + 1, GraphTexture::default);
            }

            let usage = graph.usage(*id);
            let image = &mut self.images[id.index()];

            if usage.contains(vk::ImageUsageFlags::SAMPLED) {
                let slot = bindless.allocate(BindlessBinding::Texture)?;
                bindless.write(&slot, graph.image_view(*id), graph.sampled_layout(*id));
                image.texture = Some(slot);
            }

            if usage.contains(vk::ImageUsageFlags::STORAGE) {
                image.storage_images = (0..graph.mip_levels(*id))
                    .map(|level| -> anyhow::Result<BindlessSlot> {
                        let slot = bindless.allocate(BindlessBinding::StorageImage)?;
                        bindless.write(&slot, graph.level_view(*id, level), vk::ImageLayout::GENERAL);
                        Ok(slot)
                    })
                    .collect::<anyhow::Result<_>>()?;
            }
        }

        Ok(())
    }

    // Panics for an image whose pass never declared it sampled.
    pub fn texture(&self, image: ImageId) -> u32 {
        self.images
            .get(image.index())
            .and_then(|registered| registered.texture.as_ref())
            .map(BindlessSlot::index)
            .unwrap_or_else(|| panic!("image {} is not sampled through the render graph", image.index()))
    }

    // Panics for a level whose pass never declared it a storage image.
    pub fn storage_image(&self, image: ImageId, level: u32) -> u32 {
        self.images
            .get(image.index())
            .and_then(|registered| registered.storage_images.get(level as usize))
            .map(BindlessSlot::index)
            .unwrap_or_else(|| panic!("level {level} of image {} is not a storage image", image.index()))
    }
}
