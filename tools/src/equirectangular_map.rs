use std::f32::consts::{PI, TAU};

use anyhow::Context;
use glam::Vec3;
use image::Rgb32FImage;

const PERCENTILE_SAMPLE_STRIDE: usize = 97;
const PERCENTILES: [f32; 5] = [50.0, 90.0, 99.0, 99.9, 99.99];
const LUMINANCE_WEIGHTS: Vec3 = Vec3::new(0.2126, 0.7152, 0.0722);

pub struct EquirectangularMap {
    image: Rgb32FImage,
}

impl EquirectangularMap {
    pub fn open(path: &str) -> anyhow::Result<EquirectangularMap> {
        let mut reader = image::ImageReader::open(path).with_context(|| format!("opening {path}"))?;
        reader.no_limits();
        let image = reader
            .decode()
            .with_context(|| format!("reading {path}"))?
            .into_rgb32f();

        Ok(EquirectangularMap { image })
    }

    pub fn print_brightness_percentiles(&self) {
        let mut luminance: Vec<f32> = self
            .image
            .pixels()
            .step_by(PERCENTILE_SAMPLE_STRIDE)
            .map(|pixel| Vec3::from(pixel.0).dot(LUMINANCE_WEIGHTS))
            .collect();
        luminance.sort_by(f32::total_cmp);

        for percentile in PERCENTILES {
            let index = ((percentile / 100.0) * (luminance.len() - 1) as f32) as usize;
            println!("luminance p{percentile}: {}", luminance[index]);
        }
    }

    pub fn sample(&self, direction: Vec3) -> Vec3 {
        let right_ascension = direction.y.atan2(direction.x);
        let declination = direction.z.clamp(-1.0, 1.0).asin();
        let u = (0.5 - right_ascension / TAU).rem_euclid(1.0);
        let v = 0.5 - declination / PI;

        let width = self.image.width() as usize;
        let height = self.image.height() as usize;
        let x = u * width as f32 - 0.5;
        let y = (v * height as f32 - 0.5).clamp(0.0, (height - 1) as f32);
        let x0 = x.floor();
        let y0 = y.floor();
        let fraction_x = x - x0;
        let fraction_y = y - y0;
        let texel = |x: f32, y: f32| {
            let column = (x as isize).rem_euclid(width as isize) as u32;
            let row = (y as usize).min(height - 1) as u32;

            Vec3::from(self.image.get_pixel(column, row).0)
        };
        let top = texel(x0, y0).lerp(texel(x0 + 1.0, y0), fraction_x);
        let bottom = texel(x0, y0 + 1.0).lerp(texel(x0 + 1.0, y0 + 1.0), fraction_x);

        top.lerp(bottom, fraction_y)
    }
}
