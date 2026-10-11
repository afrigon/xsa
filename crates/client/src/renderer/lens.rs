use std::path::Path;

use ash::vk;
use glam::{Vec2, Vec3};
use xsa_packs::LensDefinition;

use super::TextureHandle;
use super::gpu_data::{GhostData, ProjectedLightData};
use super::texture::{ColorSpace, DdsImage};
use crate::vulkan::{Allocator, Buffer, Device, MemoryLocation};

// The smallest scene value that can change an 8-bit pixel: half an output step, decoded through sRGB's linear segment.
// Every tonemapper darkens values near black, so none shows a smaller one.
const OUTPUT_LEVELS: f32 = 255.0;
const SRGB_LINEAR_SLOPE: f32 = 12.92;
const VISIBLE_EXPOSED: f32 = 0.5 / OUTPUT_LEVELS / SRGB_LINEAR_SLOPE;

pub(in crate::renderer) struct Lens {
    pub vignetting_exponent: f32,
    pub glare_intensity: f32,
    pub glare_falloff: f32,
    pub starburst_texture: u32,
    pub starburst_inner: f32,
    pub starburst_extent: f32,
    pub ghost_count: u32,
    // The brightest starburst value at each log-polar row's angle or beyond.
    starburst_maxima: Vec<f32>,
    // Each ghost's brightest texel times its tint.
    ghost_peaks: Vec<f32>,
    ghost_data: Vec<GhostData>,
    ghosts: Option<Buffer>,
}

impl Lens {
    pub fn new(
        definition: &LensDefinition,
        mut upload: impl FnMut(&str, &DdsImage) -> anyhow::Result<TextureHandle>,
    ) -> anyhow::Result<Lens> {
        let starburst = Lens::read(&definition.starburst.texture)?;
        let starburst_maxima = Lens::starburst_maxima(&starburst)?;
        let starburst_texture = upload("lens starburst", &starburst)?.index();
        let mut ghost_peaks = Vec::with_capacity(definition.ghosts.len());
        let mut ghost_data = Vec::with_capacity(definition.ghosts.len());

        for (index, ghost) in definition.ghosts.iter().enumerate() {
            let image = Lens::read(&ghost.texture)?;
            let peak = image
                .rgb_texels()?
                .iter()
                .fold(0.0_f32, |peak, texel| peak.max(texel.max_element()));
            ghost_peaks.push(peak * ghost.tint.max_element());
            ghost_data.push(GhostData {
                position: ghost.position,
                size: ghost.size_degrees.to_radians(),
                texture: upload(&format!("lens ghost {index}"), &image)?.index(),
                dispersion: ghost.dispersion,
                tint: ghost.tint,
                padding: 0,
            });
        }

        Ok(Lens {
            vignetting_exponent: definition.vignetting.exponent,
            glare_intensity: definition.glare.intensity,
            glare_falloff: definition.glare.falloff,
            starburst_texture,
            starburst_inner: definition.starburst.inner_degrees.to_radians(),
            starburst_extent: definition.starburst.extent_degrees.to_radians(),
            ghost_count: ghost_data.len() as u32,
            starburst_maxima,
            ghost_peaks,
            ghost_data,
            ghosts: None,
        })
    }

    fn read(path: &Path) -> anyhow::Result<DdsImage> {
        DdsImage::read(path, ColorSpace::Linear)
    }

    fn starburst_maxima(starburst: &DdsImage) -> anyhow::Result<Vec<f32>> {
        let texels = starburst.rgb_texels()?;
        let mut maxima: Vec<f32> = texels
            .chunks_exact(starburst.width() as usize)
            .map(|row| row.iter().fold(0.0_f32, |peak, texel| peak.max(texel.max_element())))
            .collect();

        for row in (0..maxima.len().saturating_sub(1)).rev() {
            maxima[row] = maxima[row].max(maxima[row + 1]);
        }

        Ok(maxima)
    }

    // Whether any part of the light's lens effects can change a pixel: an upper bound on their brightest point on
    // screen against the smallest value the output shows.
    pub fn reaches_screen(&self, light: &ProjectedLightData, exposure: f32, viewport: Vec2, focal_length: f32) -> bool {
        let scale = light.illuminance.max_element() * light.visibility * light.acceptance * exposure;
        let brightest =
            self.light_pass_bound(light, viewport, focal_length) + self.ghost_bound(light, viewport, focal_length);

        scale * brightest >= VISIBLE_EXPOSED
    }

    // The glow and starburst both fade with the angle from the light: neither exceeds its value at the screen point
    // nearest the light.
    fn light_pass_bound(&self, light: &ProjectedLightData, viewport: Vec2, focal_length: f32) -> f32 {
        let nearest = light.screen_position.clamp(Vec2::ZERO, viewport);
        let angle = Lens::pixel_ray(nearest, viewport, focal_length).angle_between(Lens::pixel_ray(
            light.screen_position,
            viewport,
            focal_length,
        ));
        let glare = self.glare_intensity / angle.max(light.angular_radius).to_degrees().powf(self.glare_falloff);

        glare + self.starburst_bound(angle)
    }

    fn starburst_bound(&self, angle: f32) -> f32 {
        if angle > self.starburst_extent {
            return 0.0;
        }

        let rows = self.starburst_maxima.len();
        let distance = (angle.max(self.starburst_inner) / self.starburst_inner).ln()
            / (self.starburst_extent / self.starburst_inner).ln();
        let row = ((distance * rows as f32) as usize).min(rows - 1);

        self.starburst_maxima[row]
    }

    // A ghost adds its peak if the circle around its quad, widened by its dispersion, touches the screen.
    fn ghost_bound(&self, light: &ProjectedLightData, viewport: Vec2, focal_length: f32) -> f32 {
        let center = viewport * 0.5;

        self.ghost_data
            .iter()
            .zip(&self.ghost_peaks)
            .filter(|(ghost, _)| {
                let ghost_center = light.screen_position.lerp(center, ghost.position);
                let half_size = (ghost.size * 0.5).tan() * focal_length * (1.0 + 2.0 * ghost.dispersion);
                let reach = half_size * std::f32::consts::SQRT_2;

                ghost_center.distance(ghost_center.clamp(Vec2::ZERO, viewport)) <= reach
            })
            .map(|(_, peak)| peak)
            .sum()
    }

    fn pixel_ray(pixel: Vec2, viewport: Vec2, focal_length: f32) -> Vec3 {
        (pixel - viewport * 0.5).extend(focal_length).normalize()
    }

    pub fn upload(&mut self, device: &Device, allocator: &mut Allocator) -> anyhow::Result<()> {
        if self.ghost_data.is_empty() {
            return Ok(());
        }

        let mut ghosts = Buffer::new(
            device,
            allocator,
            "lens ghosts",
            size_of_val(self.ghost_data.as_slice()) as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            MemoryLocation::CpuToGpu,
        )?;

        if let Err(err) = ghosts.write(&self.ghost_data) {
            unsafe { ghosts.destroy(device, allocator) };
            return Err(err);
        }

        self.ghosts = Some(ghosts);

        Ok(())
    }

    pub fn ghosts(&self) -> vk::DeviceAddress {
        self.ghosts.as_ref().map_or(0, Buffer::device_address)
    }

    // Safety: only once the GPU has finished every frame that used the lens.
    pub unsafe fn destroy(&mut self, device: &Device, allocator: &mut Allocator) {
        if let Some(ghosts) = &mut self.ghosts {
            unsafe { ghosts.destroy(device, allocator) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORT: Vec2 = Vec2::new(1920.0, 1080.0);
    const FOCAL_LENGTH: f32 = 805.5;
    // About EV100 15.6, the exposure for sunlight at Earth.
    const EXPOSURE: f32 = 1.6e-5;

    fn lens(ghosts: Vec<GhostData>, ghost_peaks: Vec<f32>) -> Lens {
        Lens {
            vignetting_exponent: 2.0,
            glare_intensity: 0.93,
            glare_falloff: 2.2,
            starburst_texture: 0,
            starburst_inner: 0.01_f32.to_radians(),
            starburst_extent: 90.0_f32.to_radians(),
            ghost_count: ghosts.len() as u32,
            starburst_maxima: vec![1e-3; 16],
            ghost_peaks,
            ghost_data: ghosts,
            ghosts: None,
        }
    }

    fn light(screen_position: Vec2, illuminance: f32) -> ProjectedLightData {
        ProjectedLightData {
            screen_position,
            visibility: 1.0,
            angular_radius: 0.0046,
            illuminance: Vec3::splat(illuminance),
            acceptance: 1.0,
        }
    }

    #[test]
    fn a_dim_light_on_screen_reaches_it() {
        assert!(lens(vec![], vec![]).reaches_screen(&light(VIEWPORT * 0.5, 1.0), EXPOSURE, VIEWPORT, FOCAL_LENGTH));
    }

    #[test]
    fn a_dim_light_far_off_screen_does_not() {
        let far_off = Vec2::new(-20_000.0, 540.0);

        assert!(!lens(vec![], vec![]).reaches_screen(&light(far_off, 1.0), EXPOSURE, VIEWPORT, FOCAL_LENGTH));
    }

    #[test]
    fn a_bright_light_just_off_screen_reaches_it_through_its_glow() {
        let just_off = Vec2::new(-200.0, 540.0);

        assert!(lens(vec![], vec![]).reaches_screen(&light(just_off, 1.3e5), EXPOSURE, VIEWPORT, FOCAL_LENGTH));
    }

    #[test]
    fn a_ghost_on_screen_keeps_a_light_its_glow_alone_would_not() {
        let far_off = Vec2::new(-20_000.0, 540.0);
        // Position 1.0 puts the ghost at the screen center whatever the light's position.
        let ghost = GhostData {
            position: 1.0,
            size: 0.1,
            texture: 0,
            dispersion: 0.0,
            tint: Vec3::ONE,
            padding: 0,
        };
        let without = lens(vec![], vec![]);
        let with = lens(vec![ghost], vec![1.0]);

        assert!(!without.reaches_screen(&light(far_off, 20.0), EXPOSURE, VIEWPORT, FOCAL_LENGTH));
        assert!(with.reaches_screen(&light(far_off, 20.0), EXPOSURE, VIEWPORT, FOCAL_LENGTH));
    }
}
