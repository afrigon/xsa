use glam::{DVec3, IVec3, Vec3};
use xsa_core::random::Seed;

// Where the camera stands in a seeded noise of `size` meters, in the body's frame: its cell and the fraction across
// it, so pixels add only their small offset in f32. Mirrored by `NoiseAnchor` in noise.slang.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(in crate::renderer) struct NoiseAnchor {
    pub cell: IVec3,
    pub seed: u32,
    pub fraction: Vec3,
    pub inverse_size: f32,
}

impl NoiseAnchor {
    pub const NONE: NoiseAnchor = NoiseAnchor {
        cell: IVec3::ZERO,
        seed: 0,
        fraction: Vec3::ZERO,
        inverse_size: 0.0,
    };

    pub fn at(point: DVec3, size: f64, seed: Seed) -> NoiseAnchor {
        let place = point / size;
        let cell = place.floor();

        NoiseAnchor {
            cell: cell.as_ivec3(),
            // The shader hashes 32 bits; the seed's low half keeps its spread.
            seed: seed.value as u32,
            fraction: (place - cell).as_vec3(),
            inverse_size: (1.0 / size) as f32,
        }
    }
}
