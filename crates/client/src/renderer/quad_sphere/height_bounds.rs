use std::f64::consts::{FRAC_PI_2, PI, TAU};

use glam::DVec3;
use xsa_core::terrain::HeightMap;

use super::HeightRange;

// Each finest bounds cell covers this many height texels on a side.
const BLOCK_TEXELS: usize = 16;
// The shader's cubic B-spline reads two texels on each side of a sample.
const SAMPLE_FOOTPRINT_TEXELS: f64 = 2.0;
// A query reads at most this many cells on a side, at the finest level that keeps it under.
const QUERY_CELLS: f64 = 4.0;

// The lowest and highest height inside each region of an equirectangular height map, and the steepest step between
// neighboring texels, coarser levels covering larger regions, so a tile's height range is a handful of reads.
pub(in crate::renderer) struct HeightBounds {
    levels: Vec<HeightBoundsLevel>,
    map_width: usize,
    map_height: usize,
    minimum: f64,
    maximum: f64,
}

struct HeightBoundsLevel {
    width: usize,
    height: usize,
    lowest: Vec<u16>,
    highest: Vec<u16>,
    // The largest difference between neighbors along a row, then along a column.
    steepest_east_west: Vec<u16>,
    steepest_north_south: Vec<u16>,
}

impl HeightBounds {
    pub fn new(map: &HeightMap) -> HeightBounds {
        let map_width = map.grid().width;
        let map_height = map.grid().height;
        let mut levels = vec![HeightBoundsLevel::from_texels(map_width, map_height, map.texels())];

        while levels.last().is_some_and(|level| level.width > 1 || level.height > 1) {
            let coarser = levels
                .last()
                .map(HeightBoundsLevel::coarser)
                .expect("levels start non-empty");
            levels.push(coarser);
        }

        HeightBounds {
            levels,
            map_width,
            map_height,
            minimum: map.minimum(),
            maximum: map.maximum(),
        }
    }

    pub fn minimum(&self) -> f64 {
        self.minimum
    }

    pub fn maximum(&self) -> f64 {
        self.maximum
    }

    // The angle between neighboring texels of the top level, along the equator.
    pub fn texel_angle(&self) -> f64 {
        TAU / self.map_width as f64
    }

    // Bounds every height the shader can sample inside the cap of `angular_radius` around `direction`, in the body's
    // frame. `sample_spacing` is the widest spacing between the cap's vertices, which coarser mip levels average over.
    // With `center_height`, the B-spline's exact height at `direction` for a cap read on the top level only, the
    // range also stays within the steepest slope of the cap's center.
    pub fn range(
        &self,
        direction: DVec3,
        angular_radius: f64,
        sample_spacing: f64,
        center_height: Option<f64>,
    ) -> HeightRange {
        let radius = angular_radius + SAMPLE_FOOTPRINT_TEXELS * self.texel_angle().max(sample_spacing);
        let latitude = direction.z.clamp(-1.0, 1.0).asin();
        let longitude = direction.y.atan2(direction.x);
        let south = (latitude - radius).max(-FRAC_PI_2);
        let north = (latitude + radius).min(FRAC_PI_2);
        let longitude_reach = (radius.sin() / latitude.cos()).min(1.0).asin();
        let reaches_pole = south == -FRAC_PI_2 || north == FRAC_PI_2;
        let covers_every_longitude = reaches_pole || longitude_reach >= FRAC_PI_2;

        // Texel coordinates: u = 0.25 − longitude / 2π grows westward, v = 0.5 + latitude / π from the south pole.
        let to_x = |longitude: f64| (0.25 - longitude / TAU) * self.map_width as f64;
        let to_y = |latitude: f64| (0.5 + latitude / PI) * self.map_height as f64;
        let (west, east) = if covers_every_longitude {
            (0.0, self.map_width as f64)
        } else {
            (to_x(longitude + longitude_reach), to_x(longitude - longitude_reach))
        };
        let (bottom, top) = (to_y(south), to_y(north));

        let level = self.query_level(east - west, top - bottom);
        let cell_texels = (BLOCK_TEXELS << level) as f64;
        let bounds = &self.levels[level];
        let first_x = (west / cell_texels).floor() as i64;
        let last_x = (east / cell_texels).floor() as i64;
        let first_y = ((bottom / cell_texels).floor() as i64).max(0) as usize;
        let last_y = ((top / cell_texels).floor() as i64).min(bounds.height as i64 - 1) as usize;
        let mut lowest = u16::MAX;
        let mut highest = u16::MIN;
        let mut steepest_east_west = u16::MIN;
        let mut steepest_north_south = u16::MIN;

        for y in first_y..=last_y {
            for x in first_x..=last_x.min(first_x + bounds.width as i64 - 1) {
                let index = y * bounds.width + x.rem_euclid(bounds.width as i64) as usize;
                lowest = lowest.min(bounds.lowest[index]);
                highest = highest.max(bounds.highest[index]);
                steepest_east_west = steepest_east_west.max(bounds.steepest_east_west[index]);
                steepest_north_south = steepest_north_south.max(bounds.steepest_north_south[index]);
            }
        }

        let table = HeightRange {
            minimum: self.height(lowest),
            maximum: self.height(highest),
        };

        let Some(center_height) = center_height.filter(|_| !reaches_pole) else {
            return table;
        };

        // The B-spline's slope never exceeds the steepest step between the texels it blends, per texel spacing.
        let poleward_cosine = south.abs().max(north.abs()).cos();
        let east_west = self.height_step(steepest_east_west) / (self.texel_angle() * poleward_cosine);
        let north_south = self.height_step(steepest_north_south) / (PI / self.map_height as f64);
        let reach = east_west.hypot(north_south) * angular_radius;

        HeightRange {
            minimum: table.minimum.max(center_height - reach),
            maximum: table.maximum.min(center_height + reach),
        }
    }

    fn query_level(&self, width_texels: f64, height_texels: f64) -> usize {
        let span = width_texels.max(height_texels) / BLOCK_TEXELS as f64;
        let level = (span / QUERY_CELLS).log2().ceil().max(0.0) as usize;

        level.min(self.levels.len() - 1)
    }

    fn height(&self, value: u16) -> f64 {
        self.minimum + self.height_step(value)
    }

    // Meters for a difference of `value` in the map's encoding.
    fn height_step(&self, value: u16) -> f64 {
        f64::from(value) / f64::from(u16::MAX) * (self.maximum - self.minimum)
    }
}

impl HeightBoundsLevel {
    fn from_texels(map_width: usize, map_height: usize, texels: &[u16]) -> HeightBoundsLevel {
        let width = map_width.div_ceil(BLOCK_TEXELS);
        let height = map_height.div_ceil(BLOCK_TEXELS);
        let mut level = HeightBoundsLevel::empty(width, height);

        for row in 0..map_height {
            let cell_row = row / BLOCK_TEXELS * width;

            for column in 0..map_width {
                let value = texels[row * map_width + column];
                let west = texels[row * map_width + (column + 1) % map_width];
                let north = texels[(row + 1).min(map_height - 1) * map_width + column];
                let index = cell_row + column / BLOCK_TEXELS;
                level.lowest[index] = level.lowest[index].min(value);
                level.highest[index] = level.highest[index].max(value);
                level.steepest_east_west[index] = level.steepest_east_west[index].max(value.abs_diff(west));
                level.steepest_north_south[index] = level.steepest_north_south[index].max(value.abs_diff(north));
            }
        }

        level
    }

    fn coarser(&self) -> HeightBoundsLevel {
        let mut level = HeightBoundsLevel::empty(self.width.div_ceil(2), self.height.div_ceil(2));

        for y in 0..self.height {
            for x in 0..self.width {
                let source = y * self.width + x;
                let index = y / 2 * level.width + x / 2;
                level.lowest[index] = level.lowest[index].min(self.lowest[source]);
                level.highest[index] = level.highest[index].max(self.highest[source]);
                level.steepest_east_west[index] = level.steepest_east_west[index].max(self.steepest_east_west[source]);
                level.steepest_north_south[index] =
                    level.steepest_north_south[index].max(self.steepest_north_south[source]);
            }
        }

        level
    }

    fn empty(width: usize, height: usize) -> HeightBoundsLevel {
        HeightBoundsLevel {
            width,
            height,
            lowest: vec![u16::MAX; width * height],
            highest: vec![u16::MIN; width * height],
            steepest_east_west: vec![u16::MIN; width * height],
            steepest_north_south: vec![u16::MIN; width * height],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAP_WIDTH: usize = 256;
    const MAP_HEIGHT: usize = 128;
    const MINIMUM: f64 = -1000.0;
    const MAXIMUM: f64 = 3000.0;
    const SMALL_CAP: f64 = 0.01;

    fn direction(latitude_degrees: f64, longitude_degrees: f64) -> DVec3 {
        let latitude = latitude_degrees.to_radians();
        let longitude = longitude_degrees.to_radians();
        DVec3::new(
            latitude.cos() * longitude.cos(),
            latitude.cos() * longitude.sin(),
            latitude.sin(),
        )
    }

    // A flat map at its middle value with one texel raised to the maximum and one lowered to the minimum.
    fn bounds(peak_x: usize, peak_y: usize, pit_x: usize, pit_y: usize) -> HeightBounds {
        let mut texels = vec![u16::MAX / 2; MAP_WIDTH * MAP_HEIGHT];
        texels[peak_y * MAP_WIDTH + peak_x] = u16::MAX;
        texels[pit_y * MAP_WIDTH + pit_x] = 0;
        HeightBounds::new(&HeightMap::new(MAP_WIDTH, MAP_HEIGHT, texels, MINIMUM, MAXIMUM).unwrap())
    }

    fn texel_direction(x: usize, y: usize) -> DVec3 {
        let u = (x as f64 + 0.5) / MAP_WIDTH as f64;
        let v = (y as f64 + 0.5) / MAP_HEIGHT as f64;
        direction((v - 0.5) * 180.0, (0.25 - u) * 360.0)
    }

    #[test]
    fn a_cap_over_a_peak_reaches_the_maximum() {
        let bounds = bounds(100, 40, 10, 100);

        assert_eq!(
            bounds.range(texel_direction(100, 40), SMALL_CAP, 0.0, None).maximum,
            MAXIMUM
        );
    }

    #[test]
    fn a_cap_away_from_the_extremes_is_tight() {
        let bounds = bounds(100, 40, 10, 100);
        let range = bounds.range(texel_direction(200, 64), SMALL_CAP, 0.0, None);

        assert!(range.minimum > MINIMUM && range.maximum < MAXIMUM);
        assert!(range.maximum - range.minimum < 1.0);
    }

    #[test]
    fn a_cap_across_the_seam_sees_both_sides() {
        let bounds = bounds(MAP_WIDTH - 1, 64, 0, 64);
        let range = bounds.range(texel_direction(0, 64), 0.05, 0.0, None);

        assert_eq!(range.minimum, MINIMUM);
        assert_eq!(range.maximum, MAXIMUM);
    }

    #[test]
    fn a_cap_over_a_pole_sees_every_longitude() {
        let bounds = bounds(3, MAP_HEIGHT - 1, MAP_WIDTH / 2, MAP_HEIGHT - 1);
        let range = bounds.range(direction(89.0, 0.0), 0.05, 0.0, None);

        assert_eq!(range.minimum, MINIMUM);
        assert_eq!(range.maximum, MAXIMUM);
    }

    #[test]
    fn the_whole_sphere_is_the_global_range() {
        let bounds = bounds(100, 40, 10, 100);
        let range = bounds.range(direction(0.0, 0.0), PI, 0.0, None);

        assert_eq!(range.minimum, MINIMUM);
        assert_eq!(range.maximum, MAXIMUM);
    }

    // A slope along the meridians: the exact center and its steepest slope pin a small cap's range down, and every
    // height inside the cap still falls within it.
    #[test]
    fn a_known_center_tightens_a_small_cap() {
        let texels: Vec<u16> = (0..MAP_WIDTH * MAP_HEIGHT)
            .map(|index| ((index / MAP_WIDTH) * 400) as u16)
            .collect();
        let map = HeightMap::new(MAP_WIDTH, MAP_HEIGHT, texels, MINIMUM, MAXIMUM).unwrap();
        let bounds = HeightBounds::new(&map);
        let center = texel_direction(30, 64);
        let cap = 0.001;
        let table = bounds.range(center, cap, 0.0, None);
        let refined = bounds.range(center, cap, 0.0, Some(map.height_at(center)));

        assert!(refined.maximum - refined.minimum < (table.maximum - table.minimum) / 4.0);

        for step in 0..16 {
            let angle = f64::from(step) / 16.0 * TAU;
            let tangent = DVec3::new(-center.y, center.x, 0.0).normalize();
            let north = center.cross(tangent);
            let edge = (center + (tangent * angle.cos() + north * angle.sin()) * cap).normalize();
            let height = map.height_at(edge);
            assert!(
                refined.minimum <= height && height <= refined.maximum,
                "{height} outside {refined:?}"
            );
        }
    }
}
