use glam::{DVec3, Vec3};
use xsa_core::random::Seed;

use crate::renderer::noise_volume::NoiseVolume;

// Where the camera stands in a seeded noise of `size` meters per cell, in the body's frame: its place in the noise
// volume, which repeats every `NoiseVolume::PERIOD_CELLS` cells, wrapped in f64 so pixels add only their small offset
// in f32. The seed shifts the place, so each use reads its own part of the volume. Mirrored by `NoiseAnchor` in
// noise.slang.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(in crate::renderer) struct NoiseAnchor {
    pub place: Vec3,
    // Periods of the volume per meter.
    pub inverse_size: f32,
}

impl NoiseAnchor {
    pub const NONE: NoiseAnchor = NoiseAnchor {
        place: Vec3::ZERO,
        inverse_size: 0.0,
    };

    pub fn at(point: DVec3, size: f64, seed: Seed) -> NoiseAnchor {
        let period = size * f64::from(NoiseVolume::PERIOD_CELLS);
        let shift = DVec3::new(
            seed.derive("x").fraction(),
            seed.derive("y").fraction(),
            seed.derive("z").fraction(),
        );
        let place = point / period + shift;

        NoiseAnchor {
            place: (place - place.floor()).as_vec3(),
            inverse_size: (1.0 / period) as f32,
        }
    }
}
