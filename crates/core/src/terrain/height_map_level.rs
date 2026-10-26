use anyhow::ensure;
use glam::DVec3;
use resource_loader::utils::CubeFace;

use super::{CubeTexel, FacePoint, TexelSource};

// The B-spline reads four texels per axis, reaching two past an edge into the adjacent face, which must hold them.
pub const SPLINE_TEXELS: i32 = 4;
const CORNER_TEXELS: f64 = 3.0;

// One mip level of a body's height map, held on the CPU: six equal-angle faces `extent` texels wide, of 16-bit values
// spanning `minimum` to `maximum`, meters relative to the body's radius. Heights are its cubic B-spline, reaching
// across face edges, mirrored by the terrain's `heights.slang` so the simulation and the picture agree on the ground.
pub struct HeightMapLevel {
    level: u32,
    extent: u32,
    // Face by face in `CubeFace` order, each row by row.
    texels: Vec<u16>,
    minimum: f64,
    maximum: f64,
}

impl HeightMapLevel {
    pub fn new(
        level: u32,
        extent: u32,
        texels: Vec<u16>,
        minimum: f64,
        maximum: f64,
    ) -> anyhow::Result<HeightMapLevel> {
        ensure!(
            extent >= SPLINE_TEXELS as u32,
            "a {extent}-texel face is narrower than the B-spline's {SPLINE_TEXELS} texels"
        );
        let expected = CubeFace::ALL.len() * (extent * extent) as usize;
        ensure!(
            texels.len() == expected,
            "six {extent}×{extent} faces hold {expected} texels, not {}",
            texels.len()
        );
        ensure!(
            minimum <= maximum,
            "the minimum height {minimum} is above the maximum {maximum}"
        );

        Ok(HeightMapLevel {
            level,
            extent,
            texels,
            minimum,
            maximum,
        })
    }

    // The mip level it is, 0 being the largest.
    pub fn level(&self) -> u32 {
        self.level
    }

    pub fn extent(&self) -> u32 {
        self.extent
    }

    pub fn minimum(&self) -> f64 {
        self.minimum
    }

    pub fn maximum(&self) -> f64 {
        self.maximum
    }

    // A face's stored values, row by row.
    pub fn face(&self, face: CubeFace) -> &[u16] {
        let size = (self.extent * self.extent) as usize;

        &self.texels[face.index() * size..][..size]
    }

    // Meters relative to the body's radius toward `direction`, a unit vector in the body's frame.
    pub fn height_at(&self, direction: DVec3) -> f64 {
        let point = FacePoint::from_direction(direction);
        let texel = point.texel_position(self.extent);
        let base = texel.floor();
        let weights_x = HeightMapLevel::spline_weights(texel.x - base.x);
        let weights_y = HeightMapLevel::spline_weights(texel.y - base.y);
        let mut sum = 0.0;

        for row in 0..SPLINE_TEXELS {
            let mut row_sum = 0.0;

            for column in 0..SPLINE_TEXELS {
                let texel = CubeTexel {
                    face: point.face,
                    column: base.x as i32 - 1 + column,
                    row: base.y as i32 - 1 + row,
                };
                row_sum += weights_x[column as usize] * self.value(texel);
            }

            sum += weights_y[row as usize] * row_sum;
        }

        self.minimum + sum * (self.maximum - self.minimum)
    }

    // A texel's value from 0 to 1 over the range, read across face edges.
    pub fn value(&self, texel: CubeTexel) -> f64 {
        match texel.source(self.extent as i32) {
            TexelSource::Stored(texel) => self.stored(texel),
            TexelSource::Corner(texels) => texels.iter().map(|&texel| self.stored(texel)).sum::<f64>() / CORNER_TEXELS,
        }
    }

    fn stored(&self, texel: CubeTexel) -> f64 {
        let index = texel.row as usize * self.extent as usize + texel.column as usize;

        f64::from(self.face(texel.face)[index]) / f64::from(u16::MAX)
    }

    // The weights of the four texels around a cubic B-spline sample at fraction `f` past the second.
    fn spline_weights(f: f64) -> [f64; SPLINE_TEXELS as usize] {
        let f2 = f * f;
        let f3 = f2 * f;

        [
            (1.0 - 3.0 * f + 3.0 * f2 - f3) / 6.0,
            (4.0 - 6.0 * f2 + 3.0 * f3) / 6.0,
            (1.0 + 3.0 * f + 3.0 * f2 - 3.0 * f3) / 6.0,
            f3 / 6.0,
        ]
    }
}

#[cfg(test)]
mod tests {
    use glam::DVec2;

    use super::*;

    const EXTENT: u32 = 16;
    const MINIMUM: f64 = -1000.0;
    const MAXIMUM: f64 = 3000.0;
    const TOLERANCE_METERS: f64 = 1e-6;
    // Two directions this close straddle an edge; the ground between them barely changes.
    const ACROSS: f64 = 1e-9;

    fn level(texel: impl Fn(CubeFace, u32, u32) -> u16) -> HeightMapLevel {
        let texels = CubeFace::ALL
            .iter()
            .flat_map(|&face| (0..EXTENT * EXTENT).map(move |index| (face, index % EXTENT, index / EXTENT)))
            .map(|(face, column, row)| texel(face, column, row))
            .collect();

        HeightMapLevel::new(0, EXTENT, texels, MINIMUM, MAXIMUM).unwrap()
    }

    // A height field that is a smooth function of direction, so each face's texels agree with their neighbours'.
    fn smooth(face: CubeFace, column: u32, row: u32) -> u16 {
        let coordinates = (DVec2::new(f64::from(column), f64::from(row)) + 0.5) / f64::from(EXTENT) * 2.0 - 1.0;
        let direction = FacePoint { face, coordinates }.direction();

        ((direction.x * 0.3 + direction.y * 0.2 + direction.z * 0.4 + 1.0) / 2.0 * f64::from(u16::MAX)) as u16
    }

    #[test]
    fn a_flat_level_is_flat_everywhere() {
        let flat = level(|_, _, _| u16::MAX);

        for direction in [
            DVec3::X,
            DVec3::Z,
            DVec3::ONE.normalize(),
            DVec3::new(0.3, -0.4, 0.5).normalize(),
        ] {
            assert!((flat.height_at(direction) - MAXIMUM).abs() < TOLERANCE_METERS);
        }
    }

    #[test]
    fn heights_are_continuous_across_edges_and_corners() {
        let smooth = level(smooth);
        let edge = DVec3::new(1.0, 0.3, 1.0).normalize();
        let corner = DVec3::ONE.normalize();

        for crossing in [edge, corner] {
            let nudge = DVec3::new(ACROSS, -ACROSS, 0.0);
            let near = smooth.height_at((crossing + nudge).normalize());
            let far = smooth.height_at((crossing - nudge).normalize());

            assert!((near - far).abs() < 1e-3, "{crossing}: {near} against {far}");
        }
    }

    #[test]
    fn heights_stay_within_the_range() {
        let noisy = level(|face, column, row| {
            if (column * 7 + row * 13 + face.index() as u32).is_multiple_of(3) {
                u16::MAX
            } else {
                0
            }
        });

        for step in 0..1000 {
            let angle = f64::from(step) * 0.037;
            let direction = DVec3::new(angle.cos(), angle.sin(), (angle * 0.3).sin()).normalize();
            let height = noisy.height_at(direction);

            assert!((MINIMUM..=MAXIMUM).contains(&height), "{height}");
        }
    }

    #[test]
    fn a_face_narrower_than_the_spline_is_rejected() {
        assert!(HeightMapLevel::new(0, 2, vec![0; 24], MINIMUM, MAXIMUM).is_err());
    }

    #[test]
    fn a_mismatched_texel_count_is_rejected() {
        assert!(HeightMapLevel::new(0, EXTENT, vec![0; 3], MINIMUM, MAXIMUM).is_err());
    }
}
