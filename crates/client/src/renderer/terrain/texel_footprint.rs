use std::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_2};

// How large a terrain map's texels look from a distance, for a body of `radius` meters seen through a lens of
// `focal_length_pixels`, and so which mip level a distance needs. Levels are continuous, 0 the finest; a map is
// `extent` texels wide per face at level 0.
#[derive(Clone, Copy)]
pub(in crate::renderer) struct TexelFootprint {
    pub radius: f64,
    pub focal_length_pixels: f64,
    // The angle between a tile's vertices on screen.
    pub vertex_spacing_angle: f64,
}

impl TexelFootprint {
    // The level whose texels are as far apart as the vertices at `distance`; mirrored by `terrain/heights.slang`.
    pub fn height_level(&self, extent: u32, distance: f64) -> f64 {
        (distance * self.vertex_spacing_angle / self.radius / TexelFootprint::texel_angle(extent)).log2()
    }

    // The level whose texels are a pixel wide at `distance`.
    pub fn layer_level(&self, extent: u32, distance: f64) -> f64 {
        (distance / (self.radius * self.focal_length_pixels) / TexelFootprint::texel_angle(extent)).log2()
    }

    // Whether some texel of level 0 is wider than a pixel at `distance`: the narrowest texels of an equal-angle face,
    // a factor of √2 below those across its middle, included.
    pub fn magnifies(&self, extent: u32, distance: f64) -> bool {
        self.layer_level(extent, distance) < FRAC_1_SQRT_2.log2()
    }

    // The angle between neighbouring texels of level 0 across a face's middle; equal-angle faces keep it within a
    // factor of 1.4 everywhere.
    fn texel_angle(extent: u32) -> f64 {
        FRAC_PI_2 / f64::from(extent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXTENT: u32 = 4096;
    const FOOTPRINT: TexelFootprint = TexelFootprint {
        radius: 1.7e6,
        focal_length_pixels: 1000.0,
        vertex_spacing_angle: 0.008,
    };

    #[test]
    fn twice_as_far_is_one_level_coarser() {
        let near = FOOTPRINT.height_level(EXTENT, 1.0e4);
        let far = FOOTPRINT.height_level(EXTENT, 2.0e4);

        assert!((far - near - 1.0).abs() < 1e-12);
    }

    #[test]
    fn a_texel_a_pixel_wide_is_level_zero() {
        let texel_meters = FRAC_PI_2 / f64::from(EXTENT) * FOOTPRINT.radius;
        let distance = texel_meters * FOOTPRINT.focal_length_pixels;

        assert!(FOOTPRINT.layer_level(EXTENT, distance).abs() < 1e-12);
        assert!(FOOTPRINT.magnifies(EXTENT, distance / 2.0));
        assert!(!FOOTPRINT.magnifies(EXTENT, distance));
    }
}
