use glam::{DVec2, DVec3};
use resource_loader::terrain::{PAGE_INTERIOR, PageAddress};

use super::{FacePoint, HeightPage, HeightSample};

// A terrain's height pages, as far as they are held: the client's stream them for drawing, the server reads them
// around vessels. Heights are a cubic B-spline over the finest page held, mirrored by the renderer's terrain geometry
// so the simulation and the picture agree on the ground.
pub trait HeightPages {
    // The finest level the terrain has.
    fn finest_level(&self) -> u32;

    // The page at `address`, or the nearest coarser page held, with its own address.
    fn page(&self, address: PageAddress) -> Option<(PageAddress, HeightPage)>;

    // Meters relative to the body's radius toward `direction`, a unit vector in the body's frame; None while no
    // page covering it is held.
    fn height_at(&self, direction: DVec3) -> Option<f64> {
        self.height_sample_at(direction).map(|sample| sample.height)
    }

    // The height toward `direction` with the level of the page it was read on.
    fn height_sample_at(&self, direction: DVec3) -> Option<HeightSample> {
        let point = FacePoint::from_direction(direction);
        let (address, page) = self.page(point.page(self.finest_level()))?;
        let page_origin = f64::from(PAGE_INTERIOR);
        let texel = point.texel_position(address.level)
            - DVec2::new(f64::from(address.key.x()), f64::from(address.key.y())) * page_origin;
        let base = texel.floor();
        let weights_x = HeightPage::spline_weights(texel.x - base.x);
        let weights_y = HeightPage::spline_weights(texel.y - base.y);
        let mut sum = 0.0;

        for (row, weight_y) in weights_y.iter().enumerate() {
            let y = base.y as i32 - 1 + row as i32;
            let mut row_sum = 0.0;

            for (column, weight_x) in weights_x.iter().enumerate() {
                row_sum += weight_x * page.height(base.x as i32 - 1 + column as i32, y);
            }

            sum += weight_y * row_sum;
        }

        Some(HeightSample {
            height: sum,
            level: address.level,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use glam::IVec2;
    use resource_loader::terrain::{HeightRange, PAGE_BORDER, PAGE_EXTENT, PageKey};
    use resource_loader::utils::CubeFace;

    use super::*;

    const TOLERANCE_METERS: f64 = 0.01;
    const RANGE: HeightRange = HeightRange {
        minimum: -100.0,
        maximum: 100.0,
    };

    struct Pages {
        finest_level: u32,
        pages: HashMap<PageAddress, HeightPage>,
    }

    impl HeightPages for Pages {
        fn finest_level(&self) -> u32 {
            self.finest_level
        }

        fn page(&self, mut address: PageAddress) -> Option<(PageAddress, HeightPage)> {
            loop {
                if let Some(page) = self.pages.get(&address) {
                    return Some((address, page.clone()));
                }

                address = PageAddress {
                    face: address.face,
                    level: address.level.checked_sub(1)?,
                    key: address.key.parent(),
                };
            }
        }
    }

    // A level 0 page of +X whose heights are `height(x, y)` at each stored texel, border included.
    fn page(height: impl Fn(i32, i32) -> f32) -> (PageAddress, HeightPage) {
        let border = PAGE_BORDER as i32;
        let values: Vec<u8> = (0..PAGE_EXTENT as i32)
            .flat_map(|row| (0..PAGE_EXTENT as i32).map(move |column| (column - border, row - border)))
            .flat_map(|(x, y)| RANGE.value(height(x, y)).to_le_bytes())
            .collect();
        let address = PageAddress {
            face: CubeFace::PositiveX,
            level: 0,
            key: PageKey::from_coordinates(0, 0),
        };

        (
            address,
            HeightPage {
                range: RANGE,
                values: Arc::from(values),
            },
        )
    }

    fn pages(finest_level: u32, held: impl IntoIterator<Item = (PageAddress, HeightPage)>) -> Pages {
        Pages {
            finest_level,
            pages: held.into_iter().collect(),
        }
    }

    #[test]
    fn nothing_held_has_no_height() {
        assert_eq!(pages(0, []).height_at(DVec3::X), None);
    }

    #[test]
    fn a_flat_page_is_flat() {
        let flat = pages(0, [page(|_, _| 42.0)]);
        let height = flat.height_at(DVec3::new(1.0, 0.2, -0.3).normalize()).unwrap();

        assert!((height - 42.0).abs() < TOLERANCE_METERS);
    }

    // A cubic B-spline reproduces a linear ramp exactly, so the height follows the texel position.
    #[test]
    fn heights_follow_the_texel_position() {
        let ramp = pages(0, [page(|x, _| x as f32 * 0.5)]);
        let point = FacePoint {
            face: CubeFace::PositiveX,
            coordinates: DVec2::new(0.3, -0.2),
        };
        let expected = point.texel_position(0).x * 0.5;

        assert!((ramp.height_at(point.direction()).unwrap() - expected).abs() < TOLERANCE_METERS);
    }

    #[test]
    fn a_range_spans_the_texels_it_covers() {
        let (_, ramp) = page(|x, y| (x + y) as f32);
        let range = ramp.range_over(IVec2::new(2, 0), IVec2::new(5, 3));

        assert!((f64::from(range.minimum) - 2.0).abs() < TOLERANCE_METERS);
        assert!((f64::from(range.maximum) - 8.0).abs() < TOLERANCE_METERS);
    }

    #[test]
    fn a_missing_finer_page_falls_back_to_its_coarser_page() {
        let coarse = pages(3, [page(|_, _| -7.0)]);

        assert!((coarse.height_at(DVec3::X).unwrap() + 7.0).abs() < TOLERANCE_METERS);
    }
}
