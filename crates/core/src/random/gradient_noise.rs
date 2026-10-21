use glam::{IVec3, UVec3, Vec3};

const PCG_MULTIPLIER: u32 = 1_664_525;
const PCG_INCREMENT: u32 = 1_013_904_223;
const SEED_SPREAD_Y: u32 = 0x9E37_79B9;
const SEED_SPREAD_Z: u32 = 0x85EB_CA6B;
const GRADIENT_BITS: u32 = 15;
const CORNERS: [IVec3; 8] = [
    IVec3::new(0, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 1, 1),
    IVec3::new(1, 0, 0),
    IVec3::new(1, 1, 0),
    IVec3::new(1, 0, 1),
    IVec3::new(1, 1, 1),
];

// Seeded gradient noise (Perlin, "Improving Noise", 2002) with quintic fades, from about -1 to 1, on a lattice of unit
// cells. Mirrored by `gradientNoise` in the shaders' noise.slang, step for step.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GradientNoise {
    pub seed: u32,
}

impl GradientNoise {
    // The noise at `fraction` across lattice cell `cell`.
    pub fn value(self, cell: IVec3, fraction: Vec3) -> f32 {
        self.blend(fraction, |corner| cell + corner)
    }

    // The same noise repeating every `period` cells along each axis.
    pub fn tiling_value(self, cell: IVec3, fraction: Vec3, period: i32) -> f32 {
        self.blend(fraction, |corner| (cell + corner).rem_euclid(IVec3::splat(period)))
    }

    fn blend(self, fraction: Vec3, lattice: impl Fn(IVec3) -> IVec3) -> f32 {
        let fade = fraction * fraction * fraction * (fraction * (fraction * 6.0 - 15.0) + 10.0);
        let dots = CORNERS.map(|corner| self.lattice_dot(lattice(corner), fraction - corner.as_vec3()));
        let along_x = [0, 1, 2, 3].map(|index| dots[index] + (dots[index + 4] - dots[index]) * fade.x);
        let along_y = [0, 2].map(|index| along_x[index] + (along_x[index + 1] - along_x[index]) * fade.y);
        along_y[0] + (along_y[1] - along_y[0]) * fade.z
    }

    // The offset's dot product with one of the directions to a cube's edge midpoints, picked from the bits of the
    // lattice point's hash.
    fn lattice_dot(self, cell: IVec3, offset: Vec3) -> f32 {
        let spread = UVec3::new(
            self.seed,
            self.seed.wrapping_mul(SEED_SPREAD_Y),
            self.seed.wrapping_mul(SEED_SPREAD_Z),
        );
        let hash = GradientNoise::pcg3d(cell.as_uvec3() ^ spread).x & GRADIENT_BITS;
        let first = if hash < 8 { offset.x } else { offset.y };
        let second = match hash {
            0..4 => offset.y,
            12 | 14 => offset.x,
            _ => offset.z,
        };
        let first = if hash & 1 == 0 { first } else { -first };
        let second = if hash & 2 == 0 { second } else { -second };
        first + second
    }

    // PCG3D (Jarzynski and Olano, 2020): three well-mixed words from three.
    fn pcg3d(value: UVec3) -> UVec3 {
        let mut value = UVec3::new(
            value.x.wrapping_mul(PCG_MULTIPLIER).wrapping_add(PCG_INCREMENT),
            value.y.wrapping_mul(PCG_MULTIPLIER).wrapping_add(PCG_INCREMENT),
            value.z.wrapping_mul(PCG_MULTIPLIER).wrapping_add(PCG_INCREMENT),
        );
        GradientNoise::mix(&mut value);
        value ^= value >> 16;
        GradientNoise::mix(&mut value);
        value
    }

    fn mix(value: &mut UVec3) {
        value.x = value.x.wrapping_add(value.y.wrapping_mul(value.z));
        value.y = value.y.wrapping_add(value.z.wrapping_mul(value.x));
        value.z = value.z.wrapping_add(value.x.wrapping_mul(value.y));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOISE: GradientNoise = GradientNoise { seed: 7 };

    #[test]
    fn vanishes_on_lattice_points() {
        assert_eq!(NOISE.value(IVec3::new(3, -2, 5), Vec3::ZERO), 0.0);
    }

    #[test]
    fn continues_across_cells() {
        let cell = IVec3::new(4, 1, -6);
        let end = NOISE.value(cell, Vec3::new(1.0, 0.25, 0.5));
        let start = NOISE.value(cell + IVec3::X, Vec3::new(0.0, 0.25, 0.5));
        assert!((end - start).abs() < 1e-6);
    }

    #[test]
    fn tiles_with_its_period() {
        let fraction = Vec3::new(0.3, 0.6, 0.9);
        let period = 8;
        let first = NOISE.tiling_value(IVec3::new(7, 2, 3), fraction, period);
        let wrapped = NOISE.tiling_value(IVec3::new(-1, 10, 3), fraction, period);
        assert_eq!(first, wrapped);
    }

    #[test]
    fn stays_within_its_range() {
        for step in 0..1000 {
            let fraction = Vec3::new(step as f32 * 0.37, step as f32 * 0.61, step as f32 * 0.83).fract();
            assert!(NOISE.value(IVec3::new(step, -step, 2 * step), fraction).abs() <= 2.0);
        }
    }
}
