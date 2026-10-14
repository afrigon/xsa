use std::f64::consts::{PI, TAU};

use glam::{DVec2, DVec3};

// The texel grid of an equirectangular map: longitude 0 a quarter of the way along each row, longitudes growing
// westward, rows from the south pole.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EquirectangularGrid {
    pub width: usize,
    pub height: usize,
}

impl EquirectangularGrid {
    // Texel centers sit at whole coordinates. `direction` is a unit vector in the body's frame.
    pub fn texel_position(self, direction: DVec3) -> DVec2 {
        let longitude = direction.y.atan2(direction.x);
        let latitude = direction.z.clamp(-1.0, 1.0).asin();

        DVec2::new(
            (0.25 - longitude / TAU) * self.width as f64 - 0.5,
            (0.5 + latitude / PI) * self.height as f64 - 0.5,
        )
    }
}
