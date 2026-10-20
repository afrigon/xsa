use std::sync::OnceLock;

use anyhow::ensure;
use glam::{DVec2, Vec2, Vec3};
use resource_vulkan::{ResourceHandle, TextureArray};
use xsa_core::random::Seed;

use super::{SurfaceClass, SurfaceScale};
use crate::renderer::gpu_data::{MAX_SURFACE_SCALES, SurfaceData};

const AVERAGE_COLOR_KEY: &str = "xsa.averageColor";

// One class's surface: its three texture arrays, its size and displacement range (m), and its scale ladder, which
// doubles from its own size up to its largest magnification.
pub struct SurfaceTextures {
    pub color: ResourceHandle<TextureArray>,
    pub normal: ResourceHandle<TextureArray>,
    pub displacement: ResourceHandle<TextureArray>,
    pub size: f32,
    pub displacement_low: f32,
    pub displacement_high: f32,
    pub(in crate::renderer) scales: Vec<SurfaceScale>,
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
        let scale_count = definition.largest_magnification.log2().floor() as usize + 1;
        ensure!(
            scale_count <= MAX_SURFACE_SCALES,
            "a largest magnification of {} needs {scale_count} scales, more than the {MAX_SURFACE_SCALES} drawn",
            definition.largest_magnification
        );
        let class_seed = seed.derive(class.name());

        Ok(SurfaceTextures {
            color,
            normal,
            displacement,
            size: definition.size,
            displacement_low: definition.displacement_low,
            displacement_high: definition.displacement_high,
            scales: (0..scale_count)
                .map(|scale| SurfaceScale::new(class_seed.derive(&format!("scale-{scale}"))))
                .collect(),
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
        let mut scale_rotations = [Vec2::ZERO; MAX_SURFACE_SCALES];

        for (rotation, scale) in scale_rotations.iter_mut().zip(&self.scales) {
            *rotation = scale.rotation_data();
        }

        Some(SurfaceData {
            color_texture: color.index(),
            normal_texture: normal.index(),
            displacement_texture: displacement.index(),
            variant_count: color.layer_count(),
            size: self.size,
            texels: color.width() as f32,
            displacement_low: self.displacement_low,
            displacement_high: self.displacement_high,
            average_color,
            scale_count: self.scales.len() as u32,
            scale_rotations,
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

    // Where `point` (m, on a projection plane) falls within each scale's repeat.
    pub(in crate::renderer) fn anchors(&self, point: DVec2) -> [Vec2; MAX_SURFACE_SCALES] {
        let mut anchors = [Vec2::ZERO; MAX_SURFACE_SCALES];
        let mut repeat = f64::from(self.size);

        for (anchor, scale) in anchors.iter_mut().zip(&self.scales) {
            *anchor = scale.anchor(point, repeat);
            repeat *= 2.0;
        }

        anchors
    }
}
