use std::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_2};

use resource_loader::terrain::PAGE_INTERIOR;

// How fine a terrain's pages need to be at a distance, for a body of `radius` meters seen through a lens of
// `focal_length_pixels`. Levels are continuous: level n is the level n pages, finer as it grows.
pub(in crate::renderer) struct PageDetail {
    pub radius: f64,
    pub focal_length_pixels: f64,
    // The angle between a tile's vertices on screen, which the shader picks height levels from.
    pub vertex_spacing_angle: f64,
}

impl PageDetail {
    // The angle between neighboring texels of level 0, across a face's middle; equal-angle faces keep it within
    // a factor of 1.4 everywhere.
    const LEVEL_ZERO_TEXEL_ANGLE: f64 = FRAC_PI_2 / PAGE_INTERIOR as f64;

    // The level whose texels are as far apart as the vertices at `distance`; mirrored by the terrain geometry.
    pub fn height_level(&self, distance: f64) -> f64 {
        (PageDetail::LEVEL_ZERO_TEXEL_ANGLE * self.radius / (distance * self.vertex_spacing_angle)).log2()
    }

    // The level whose texels are a pixel wide at `distance`.
    pub fn layer_level(&self, distance: f64) -> f64 {
        (self.texel_pixels(0, distance)).log2()
    }

    // How many pixels wide a texel of `level` is at `distance`: the error, in pixels, of showing its coarser page
    // instead, whose texels are twice as wide.
    pub fn texel_pixels(&self, level: u32, distance: f64) -> f64 {
        PageDetail::LEVEL_ZERO_TEXEL_ANGLE / f64::from(level).exp2() * self.radius * self.focal_length_pixels / distance
    }

    // The ground a pixel covers at `distance` (m), seen face on.
    pub fn pixel_footprint(&self, distance: f64) -> f64 {
        distance / self.focal_length_pixels
    }

    // Whether some texel of `level` is wider than a pixel at `distance`: the narrowest texels of an equal-angle face,
    // a factor of √2 below those across its middle, included.
    pub fn magnifies(&self, level: u32, distance: f64) -> bool {
        self.texel_pixels(level, distance) * FRAC_1_SQRT_2 > 1.0
    }
}
