use glam::Vec3;

use crate::equirectangular_map::EquirectangularMap;

pub const CUBE_FACE_COUNT: usize = 6;
pub const FACE_SIZE: usize = 4096;

const SAMPLES_PER_AXIS: usize = 2;

pub struct CubeFace {
    pub texels: Vec<Vec3>,
}

impl CubeFace {
    pub fn project(map: &EquirectangularMap, face: usize) -> CubeFace {
        let mut texels = Vec::with_capacity(FACE_SIZE * FACE_SIZE);
        let sample_offset = |sample: usize| (sample as f32 + 0.5) / SAMPLES_PER_AXIS as f32;

        for row in 0..FACE_SIZE {
            for column in 0..FACE_SIZE {
                let mut sum = Vec3::ZERO;

                for sample_row in 0..SAMPLES_PER_AXIS {
                    for sample_column in 0..SAMPLES_PER_AXIS {
                        let s = (column as f32 + sample_offset(sample_column)) / FACE_SIZE as f32;
                        let t = (row as f32 + sample_offset(sample_row)) / FACE_SIZE as f32;
                        sum += map.sample(CubeFace::direction(face, s * 2.0 - 1.0, t * 2.0 - 1.0));
                    }
                }

                texels.push(sum / (SAMPLES_PER_AXIS * SAMPLES_PER_AXIS) as f32);
            }
        }

        CubeFace { texels }
    }

    pub fn mip_chain(self) -> Vec<CubeFace> {
        let mut chain = vec![self];
        let mut size = FACE_SIZE;

        while size > 1 {
            let previous = &chain[chain.len() - 1].texels;
            let half = size / 2;
            let mut texels = Vec::with_capacity(half * half);

            for row in 0..half {
                for column in 0..half {
                    let at = |r: usize, c: usize| previous[(row * 2 + r) * size + column * 2 + c];
                    texels.push((at(0, 0) + at(0, 1) + at(1, 0) + at(1, 1)) / 4.0);
                }
            }

            chain.push(CubeFace { texels });
            size = half;
        }

        chain
    }

    // Inverse of the cube map face selection table in the Vulkan specification.
    fn direction(face: usize, s: f32, t: f32) -> Vec3 {
        match face {
            0 => Vec3::new(1.0, -t, -s),
            1 => Vec3::new(-1.0, -t, s),
            2 => Vec3::new(s, 1.0, t),
            3 => Vec3::new(s, -1.0, -t),
            4 => Vec3::new(s, -t, 1.0),
            _ => Vec3::new(-s, -t, -1.0),
        }
        .normalize()
    }
}
