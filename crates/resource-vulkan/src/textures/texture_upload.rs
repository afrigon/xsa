use std::ops::Range;

use anyhow::ensure;
use ash::vk;
use memory_vulkan::{
    BindlessBinding, Image, ImageCopy, ImageDescription, ImageKind, ImageMemory, ImageUpload, TexelBlock,
};
use resource_loader::texture::{TextureImage, TextureKind};

use crate::LoadContext;
use crate::textures::PendingTexture;

// Creates a texture's image, takes its bindless slot and queues the upload of the levels the decoded texture holds, in
// every layer.
pub(crate) struct TextureUpload {
    image: TextureImage,
}

impl TextureUpload {
    pub fn new(image: TextureImage) -> TextureUpload {
        TextureUpload { image }
    }

    pub fn cube_map(self, context: &LoadContext) -> anyhow::Result<PendingTexture> {
        let levels = self.image.levels();

        self.cube_map_levels(context, levels)
    }

    // A cube map image of `levels`, which a later image of other levels copies from or into: it uploads the levels
    // the decoded texture holds, the first ones of `levels`, and leaves the rest for that copy.
    pub fn cube_map_levels(self, context: &LoadContext, levels: Range<u32>) -> anyhow::Result<PendingTexture> {
        ensure!(
            self.image.kind() == TextureKind::CubeMap,
            "expected a cube map, found a {}",
            self.image.kind()
        );
        ensure!(
            self.image.width() == self.image.height(),
            "cube map faces must be square"
        );

        self.upload(context, BindlessBinding::CubeMap, ImageKind::Cube, levels)
    }

    // A cube map's faces as a six-layer array of `levels`, read texel by texel rather than sampled by direction; the
    // levels beyond the decoded ones are left for a copy, as `cube_map_levels` leaves them.
    pub fn faces(self, context: &LoadContext, levels: Range<u32>) -> anyhow::Result<PendingTexture> {
        ensure!(
            self.image.kind() == TextureKind::CubeMap,
            "expected a cube map, found a {}",
            self.image.kind()
        );
        let kind = ImageKind::Array {
            count: self.image.layers().len() as u32,
        };

        self.upload(context, BindlessBinding::TextureArray, kind, levels)
    }

    pub fn array(self, context: &LoadContext) -> anyhow::Result<PendingTexture> {
        ensure!(
            self.image.kind() == TextureKind::Array,
            "expected a texture array, found a {}",
            self.image.kind()
        );
        let kind = ImageKind::Array {
            count: self.image.layers().len() as u32,
        };

        let levels = self.image.levels();
        self.upload(context, BindlessBinding::TextureArray, kind, levels)
    }

    pub fn texture(self, context: &LoadContext) -> anyhow::Result<PendingTexture> {
        ensure!(
            self.image.kind() == TextureKind::Plane,
            "expected a 2D texture, found a {}",
            self.image.kind()
        );

        let levels = self.image.levels();
        self.upload(context, BindlessBinding::Texture, ImageKind::Single, levels)
    }

    fn upload(
        self,
        context: &LoadContext,
        binding: BindlessBinding,
        kind: ImageKind,
        levels: Range<u32>,
    ) -> anyhow::Result<PendingTexture> {
        ensure!(
            self.image.levels().start == levels.start && self.image.levels().end <= levels.end,
            "the image's levels {levels:?} start with the {:?} decoded",
            self.image.levels()
        );
        let format = self.image.format();
        let image = Image::new(
            context.memory,
            &ImageDescription {
                name: context.name,
                extent: vk::Extent2D {
                    width: (self.image.width() >> levels.start).max(1),
                    height: (self.image.height() >> levels.start).max(1),
                },
                format: vk::Format::from_raw(format.vulkan_format().value as i32),
                usage: vk::ImageUsageFlags::SAMPLED
                    | vk::ImageUsageFlags::TRANSFER_SRC
                    | vk::ImageUsageFlags::TRANSFER_DST,
                aspect: vk::ImageAspectFlags::COLOR,
                mip_levels: levels.end - levels.start,
                kind,
                memory: ImageMemory::Shared,
                sharing: context.transfer.sharing(),
            },
        )?;
        let slot = context.bindless.allocate(binding)?;
        let copies = self.copies(levels.start);

        if copies.is_empty() {
            let ticket =
                context
                    .transfer
                    .prepare_image(image, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, context.priority);

            return Ok(PendingTexture::new(ticket.into(), slot));
        }

        let upload = ImageUpload {
            data: self
                .image
                .into_layers()
                .into_iter()
                .map(|layer| Box::new(layer) as Box<dyn AsRef<[u8]> + Send + Sync>)
                .collect(),
            copies,
            texel_block: TexelBlock {
                extent: vk::Extent2D {
                    width: format.block_extent(),
                    height: format.block_extent(),
                },
                size: format.block_size() as u32,
            },
        };
        let ticket = context.transfer.upload_image(image, upload, context.priority);

        Ok(PendingTexture::new(ticket.into(), slot))
    }

    // Each layer holds its levels, largest first, packed without padding; the image's level 0 is `first_level`.
    fn copies(&self, first_level: u32) -> Vec<ImageCopy> {
        let format = self.image.format();
        let mut copies = Vec::new();

        for layer in 0..self.image.layers().len() {
            let mut offset = 0;

            for level in self.image.levels() {
                let width = (self.image.width() >> level).max(1);
                let height = (self.image.height() >> level).max(1);
                copies.push(ImageCopy {
                    data: layer,
                    offset,
                    mip_level: level - first_level,
                    array_layer: layer as u32,
                    image_offset: vk::Offset3D::default(),
                    extent: vk::Extent2D { width, height },
                });
                offset += format.level_size(width, height);
            }
        }

        copies
    }
}
