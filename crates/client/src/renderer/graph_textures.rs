use ash::vk;
use render_graph::{ImageId, RenderGraph};

use crate::vulkan::{BindlessTextures, Device};

#[derive(Default)]
struct GraphTexture {
    texture: Option<u32>,
    storage_images: Vec<u32>,
}

// The bindless slots of the images the render graph allocates, by image id. A recreated image keeps its id, so its
// slots are rewritten instead of added again.
#[derive(Default)]
pub(super) struct GraphTextures {
    images: Vec<GraphTexture>,
}

impl GraphTextures {
    pub fn register(
        &mut self,
        device: &Device,
        bindless: &mut BindlessTextures,
        graph: &RenderGraph,
        ids: &[ImageId],
    ) -> anyhow::Result<()> {
        for id in ids {
            if self.images.len() <= id.index() {
                self.images.resize_with(id.index() + 1, GraphTexture::default);
            }

            let usage = graph.usage(*id);
            let image = &mut self.images[id.index()];

            if usage.contains(vk::ImageUsageFlags::SAMPLED) {
                let view = graph.image_view(*id);
                let layout = graph.sampled_layout(*id);

                match image.texture {
                    Some(index) => bindless.set_texture(device, index, view, layout),
                    None => image.texture = Some(bindless.add_texture(device, view, layout)?),
                }
            }

            if usage.contains(vk::ImageUsageFlags::STORAGE) {
                for level in 0..graph.mip_levels(*id) {
                    let view = graph.level_view(*id, level);

                    match image.storage_images.get(level as usize) {
                        Some(index) => bindless.set_storage_image(device, *index, view),
                        None => image.storage_images.push(bindless.add_storage_image(device, view)?),
                    }
                }
            }
        }

        Ok(())
    }

    // Panics for an image whose pass never declared it sampled.
    pub fn texture(&self, image: ImageId) -> u32 {
        self.images
            .get(image.index())
            .and_then(|registered| registered.texture)
            .unwrap_or_else(|| panic!("image {} is not sampled through the render graph", image.index()))
    }

    // Panics for a level whose pass never declared it a storage image.
    pub fn storage_image(&self, image: ImageId, level: u32) -> u32 {
        self.images
            .get(image.index())
            .and_then(|registered| registered.storage_images.get(level as usize).copied())
            .unwrap_or_else(|| panic!("level {level} of image {} is not a storage image", image.index()))
    }
}
