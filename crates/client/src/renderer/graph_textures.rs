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

    pub fn texture(&self, image: Option<ImageId>) -> anyhow::Result<u32> {
        image
            .and_then(|id| self.images.get(id.index()))
            .and_then(|image| image.texture)
            .ok_or_else(|| anyhow::anyhow!("the image is not sampled through the render graph"))
    }

    pub fn storage_image(&self, image: Option<ImageId>, level: u32) -> anyhow::Result<u32> {
        image
            .and_then(|id| self.images.get(id.index()))
            .and_then(|image| image.storage_images.get(level as usize).copied())
            .ok_or_else(|| anyhow::anyhow!("level {level} of the image is not a render graph storage image"))
    }
}
