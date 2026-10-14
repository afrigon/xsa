use anyhow::ensure;
use glam::DVec3;

use super::EquirectangularGrid;

const SPLINE_TEXELS: usize = 4;

// An equirectangular map whose values span `minimum` to `maximum`, meters relative to the body's radius. Its cubic
// B-spline mirrors the renderer's `bSplineSample`, so the simulation and the picture agree on the ground.
pub struct HeightMap {
    grid: EquirectangularGrid,
    texels: Vec<u16>,
    minimum: f64,
    maximum: f64,
}

impl HeightMap {
    // `texels` run row by row in the grid's layout.
    pub fn new(width: usize, height: usize, texels: Vec<u16>, minimum: f64, maximum: f64) -> anyhow::Result<HeightMap> {
        ensure!(
            texels.len() == width * height,
            "a {width}×{height} height map needs {} texels, not {}",
            width * height,
            texels.len()
        );
        ensure!(
            minimum <= maximum,
            "the minimum height {minimum} is above the maximum {maximum}"
        );

        Ok(HeightMap {
            grid: EquirectangularGrid { width, height },
            texels,
            minimum,
            maximum,
        })
    }

    pub fn grid(&self) -> EquirectangularGrid {
        self.grid
    }

    pub fn texels(&self) -> &[u16] {
        &self.texels
    }

    pub fn minimum(&self) -> f64 {
        self.minimum
    }

    pub fn maximum(&self) -> f64 {
        self.maximum
    }

    // `direction` is a unit vector in the body's frame.
    pub fn height_at(&self, direction: DVec3) -> f64 {
        let texel = self.grid.texel_position(direction);
        let base = texel.floor();
        let weights_x = HeightMap::spline_weights(texel.x - base.x);
        let weights_y = HeightMap::spline_weights(texel.y - base.y);
        let width = self.grid.width;
        let height = self.grid.height;
        let mut sum = 0.0;

        for (row, weight_y) in weights_y.iter().enumerate() {
            let y = (base.y as i64 - 1 + row as i64).clamp(0, height as i64 - 1) as usize;
            let mut row_sum = 0.0;

            for (column, weight_x) in weights_x.iter().enumerate() {
                let x = (base.x as i64 - 1 + column as i64).rem_euclid(width as i64) as usize;
                row_sum += weight_x * f64::from(self.texels[y * width + x]);
            }

            sum += weight_y * row_sum;
        }

        self.minimum + sum / f64::from(u16::MAX) * (self.maximum - self.minimum)
    }

    fn spline_weights(f: f64) -> [f64; SPLINE_TEXELS] {
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
    use std::f64::consts::{PI, TAU};

    use super::*;

    const WIDTH: usize = 64;
    const HEIGHT: usize = 32;
    const MINIMUM: f64 = -1000.0;
    const MAXIMUM: f64 = 3000.0;
    const TOLERANCE_METERS: f64 = 1e-6;

    fn map(texel: impl Fn(usize, usize) -> u16) -> HeightMap {
        let texels = (0..WIDTH * HEIGHT)
            .map(|index| texel(index % WIDTH, index / WIDTH))
            .collect();
        HeightMap::new(WIDTH, HEIGHT, texels, MINIMUM, MAXIMUM).unwrap()
    }

    // The direction through the center of texel (x, y).
    fn texel_direction(x: f64, y: f64) -> DVec3 {
        let longitude = (0.25 - (x + 0.5) / WIDTH as f64) * TAU;
        let latitude = ((y + 0.5) / HEIGHT as f64 - 0.5) * PI;
        DVec3::new(
            latitude.cos() * longitude.cos(),
            latitude.cos() * longitude.sin(),
            latitude.sin(),
        )
    }

    #[test]
    fn a_flat_map_is_flat_everywhere() {
        let flat = map(|_, _| u16::MAX);

        for direction in [DVec3::X, DVec3::Z, -DVec3::Z, DVec3::new(0.3, -0.4, 0.5).normalize()] {
            assert!((flat.height_at(direction) - MAXIMUM).abs() < TOLERANCE_METERS);
        }
    }

    #[test]
    fn a_linear_slope_is_reproduced_exactly() {
        let slope = map(|_, y| (y * 1000) as u16);
        let step = (MAXIMUM - MINIMUM) * 1000.0 / f64::from(u16::MAX);

        for y in 2..HEIGHT - 2 {
            let expected = MINIMUM + step * y as f64;
            assert!((slope.height_at(texel_direction(10.0, y as f64)) - expected).abs() < TOLERANCE_METERS);
            assert!(
                (slope.height_at(texel_direction(10.0, y as f64 + 0.5)) - (expected + step / 2.0)).abs()
                    < TOLERANCE_METERS
            );
        }
    }

    #[test]
    fn the_seam_is_continuous() {
        let ridge = map(|x, _| if x == 0 || x == WIDTH - 1 { u16::MAX } else { 0 });
        let west = ridge.height_at(texel_direction(WIDTH as f64 - 0.5 - 1e-9, 16.0));
        let east = ridge.height_at(texel_direction(-0.5 + 1e-9, 16.0));

        assert!((west - east).abs() < TOLERANCE_METERS * 1000.0);
        assert!(west > MINIMUM);
    }

    #[test]
    fn heights_stay_within_the_range() {
        let noisy = map(|x, y| if (x * 7 + y * 13) % 3 == 0 { u16::MAX } else { 0 });

        for step in 0..1000 {
            let angle = step as f64 * 0.037;
            let direction = DVec3::new(angle.cos(), angle.sin(), (angle * 0.3).sin()).normalize();
            let height = noisy.height_at(direction);
            assert!((MINIMUM..=MAXIMUM).contains(&height), "{height}");
        }
    }

    #[test]
    fn a_mismatched_texel_count_is_rejected() {
        assert!(HeightMap::new(WIDTH, HEIGHT, vec![0; 3], MINIMUM, MAXIMUM).is_err());
    }
}
