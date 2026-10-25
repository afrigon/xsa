use std::thread::JoinHandle;

use anyhow::Context;
use ash::vk;
use glam::{UVec3, Vec3};
use memory_vulkan::{
    BindlessBinding, BindlessSlot, GpuUpload, Image, ImageCopy, ImageDescription, ImageKind, ImageMemory, ImageUpload,
    TexelBlock, TransferPriority,
};
use xsa_core::random::GradientNoise;

use super::GpuContext;

const TEXELS_PER_CELL: u32 = 8;
const SIDE: u32 = NoiseVolume::PERIOD_CELLS as u32 * TEXELS_PER_CELL;
const NOISE: GradientNoise = GradientNoise { seed: 0 };
const SNORM_SCALE: f32 = i8::MAX as f32;

// A smooth seeded noise repeating every `PERIOD_CELLS` cells along each axis, baked into a volume so shaders read it in
// one filtered fetch instead of hashing its lattice at every pixel. Each use offsets into it by its own seed. Built
// on a thread of its own, then uploaded; its slot is readable once `index` returns it.
pub(in crate::renderer) struct NoiseVolume {
    generation: Option<JoinHandle<Vec<u8>>>,
    upload: Option<GpuUpload<Image>>,
    slot: BindlessSlot,
    ready: bool,
}

impl NoiseVolume {
    pub const PERIOD_CELLS: i32 = 16;

    pub fn new(gpu: &GpuContext) -> anyhow::Result<NoiseVolume> {
        Ok(NoiseVolume {
            generation: Some(std::thread::spawn(NoiseVolume::texels)),
            upload: None,
            slot: gpu.bindless.allocate(BindlessBinding::Volume)?,
            ready: false,
        })
    }

    // Its bindless index, once uploaded.
    pub fn index(&self) -> Option<u32> {
        self.ready.then(|| self.slot.index())
    }

    pub fn poll(&mut self, gpu: &GpuContext) -> anyhow::Result<()> {
        if self.generation.as_ref().is_some_and(JoinHandle::is_finished) {
            let texels = self
                .generation
                .take()
                .expect("a finished generation is held")
                .join()
                .map_err(|_| anyhow::anyhow!("generating the noise volume panicked"))?;
            self.upload = Some(NoiseVolume::upload(gpu, texels)?);
        }

        if let Some(upload) = &mut self.upload
            && !self.ready
            && let Some(image) = upload.poll().context("uploading the noise volume")?
        {
            gpu.bindless
                .write(&self.slot, image.view(), vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
            self.ready = true;
        }

        Ok(())
    }

    fn upload(gpu: &GpuContext, texels: Vec<u8>) -> anyhow::Result<GpuUpload<Image>> {
        let image = Image::new(
            &gpu.memory,
            &ImageDescription {
                name: "noise volume",
                extent: vk::Extent2D {
                    width: SIDE,
                    height: SIDE,
                },
                format: vk::Format::R8_SNORM,
                usage: vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
                aspect: vk::ImageAspectFlags::COLOR,
                mip_levels: 1,
                kind: ImageKind::Volume { depth: SIDE },
                memory: ImageMemory::Shared,
                sharing: gpu.transfer.sharing(),
            },
        )?;
        let slice_size = (SIDE * SIDE) as usize;
        let upload = ImageUpload {
            data: vec![Box::new(texels)],
            copies: (0..SIDE)
                .map(|slice| ImageCopy {
                    data: 0,
                    offset: slice as usize * slice_size,
                    mip_level: 0,
                    array_layer: 0,
                    image_offset: vk::Offset3D {
                        x: 0,
                        y: 0,
                        z: slice as i32,
                    },
                    extent: vk::Extent2D {
                        width: SIDE,
                        height: SIDE,
                    },
                })
                .collect(),
            texel_block: TexelBlock {
                extent: vk::Extent2D { width: 1, height: 1 },
                size: 1,
            },
        };

        Ok(gpu
            .transfer
            .upload_image(image, upload, TransferPriority::Critical)
            .into())
    }

    // Texel centres sit half a texel in from each cell's subdivisions, so filtering between them follows the noise.
    fn texels() -> Vec<u8> {
        let mut texels = Vec::with_capacity((SIDE * SIDE * SIDE) as usize);

        for z in 0..SIDE {
            for y in 0..SIDE {
                for x in 0..SIDE {
                    let texel = UVec3::new(x, y, z);
                    let place = (texel.as_vec3() + Vec3::splat(0.5)) / TEXELS_PER_CELL as f32;
                    let cell = (texel / TEXELS_PER_CELL).as_ivec3();
                    let value = NOISE.tiling_value(cell, place - cell.as_vec3(), NoiseVolume::PERIOD_CELLS);
                    texels.push((value * SNORM_SCALE).round().clamp(-SNORM_SCALE, SNORM_SCALE) as i8 as u8);
                }
            }
        }

        texels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_every_texel() {
        assert_eq!(NoiseVolume::texels().len(), (SIDE * SIDE * SIDE) as usize);
    }

    #[test]
    fn spreads_over_both_signs() {
        let texels = NoiseVolume::texels();
        assert!(texels.iter().any(|&texel| (texel as i8) < -SNORM_SCALE as i8 / 2));
        assert!(texels.iter().any(|&texel| (texel as i8) > SNORM_SCALE as i8 / 2));
    }
}
