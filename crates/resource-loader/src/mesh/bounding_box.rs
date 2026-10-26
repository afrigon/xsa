use glam::Vec3;

/// The box aligned with a model's axes enclosing every point of its mesh, in the model's frame.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BoundingBox {
    pub minimum: Vec3,
    pub maximum: Vec3,
}

impl BoundingBox {
    /// An empty box at the origin without points.
    pub fn around(points: impl Iterator<Item = Vec3>) -> BoundingBox {
        let (minimum, maximum) = points.fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(minimum, maximum), point| {
            (minimum.min(point), maximum.max(point))
        });

        if minimum.x > maximum.x {
            return BoundingBox {
                minimum: Vec3::ZERO,
                maximum: Vec3::ZERO,
            };
        }

        BoundingBox { minimum, maximum }
    }

    pub fn center(self) -> Vec3 {
        (self.minimum + self.maximum) / 2.0
    }

    pub fn half_extent(self) -> Vec3 {
        (self.maximum - self.minimum) / 2.0
    }

    /// The radius of the sphere around the box's center through its corners.
    pub fn radius(self) -> f32 {
        self.half_extent().length()
    }
}
