use std::sync::OnceLock;

use anyhow::ensure;
use glam::{DVec2, Vec2, Vec3};
use resource_vulkan::{ResourceHandle, TextureArray};
use xsa_core::random::Seed;

use super::{SURFACE_LEVEL_GROWTH, SurfaceClass};
use crate::renderer::gpu_data::SurfaceData;

const AVERAGE_COLOR_KEY: &str = "xsa.averageColor";

// One class's surface: its three texture arrays, and its size and displacement range (m).
pub struct SurfaceTextures {
    pub color: ResourceHandle<TextureArray>,
    pub normal: ResourceHandle<TextureArray>,
    pub displacement: ResourceHandle<TextureArray>,
    pub size: f32,
    pub displacement_low: f32,
    pub displacement_high: f32,
    tiling_seed: Seed,
    // Read from the colour array's metadata once it is loaded; None when the metadata is unusable.
    average_color: OnceLock<Option<Vec3>>,
}

impl SurfaceTextures {
    pub fn new(
        color: ResourceHandle<TextureArray>,
        normal: ResourceHandle<TextureArray>,
        displacement: ResourceHandle<TextureArray>,
        definition: &xsa_packs::SurfaceDefinition,
        class: SurfaceClass,
        seed: Seed,
    ) -> anyhow::Result<SurfaceTextures> {
        let class_seed = seed.derive(class.name());

        Ok(SurfaceTextures {
            color,
            normal,
            displacement,
            size: definition.size,
            displacement_low: definition.displacement_low,
            displacement_high: definition.displacement_high,
            tiling_seed: class_seed.derive("tiling"),
            average_color: OnceLock::new(),
        })
    }

    // None until its three arrays are loaded, and for good when its colour array's metadata is unusable.
    pub(in crate::renderer) fn gpu_data(&self) -> Option<SurfaceData> {
        let (Some(color), Some(normal), Some(displacement)) =
            (self.color.get(), self.normal.get(), self.displacement.get())
        else {
            return None;
        };
        let average_color = (*self.average_color.get_or_init(|| {
            SurfaceTextures::read_average_color(color)
                .inspect_err(|err| tracing::error!("a surface cannot be drawn: {err:#}"))
                .ok()
        }))?;
        Some(SurfaceData {
            color_texture: color.index(),
            normal_texture: normal.index(),
            displacement_texture: displacement.index(),
            // The shader hashes 32 bits; the seed's low half keeps its spread.
            tiling_seed: self.tiling_seed.value as u32,
            size: self.size,
            texels: color.width() as f32,
            displacement_low: self.displacement_low,
            displacement_high: self.displacement_high,
            average_color,
            padding: [0; 5],
        })
    }

    fn read_average_color(color: &TextureArray) -> anyhow::Result<Vec3> {
        let average = color.metadata().numbers(AVERAGE_COLOR_KEY)?;
        ensure!(
            average.len() == 3,
            "{AVERAGE_COLOR_KEY} holds {} numbers, not 3",
            average.len()
        );

        Ok(Vec3::from_slice(&average))
    }

    // Where `point` (m, on a projection plane) falls within its repeat at `level`, in repeats.
    pub(in crate::renderer) fn repeat_place(&self, point: DVec2, level: usize) -> Vec2 {
        let place = point / self.repeat(level);
        (place - place.floor()).as_vec2()
    }

    fn repeat(&self, level: usize) -> f64 {
        f64::from(self.size) * SURFACE_LEVEL_GROWTH.powi(level as i32)
    }
}
