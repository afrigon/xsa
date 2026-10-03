use glam::DVec3;

use super::view_point::ViewPoint;
use super::zoom_path_shape::ZoomPathShape;

// Smooth zoom-and-pan path from van Wijk & Nuij, "Smooth and efficient zooming and panning" (2003).
pub(super) struct ZoomPath {
    start: ViewPoint,
    offset: DVec3,
    curvature: f64,
    shape: ZoomPathShape,
    pub length: f64,
}

impl ZoomPath {
    pub fn new(start: ViewPoint, end: ViewPoint, curvature: f64) -> Self {
        let offset = end.focus - start.focus;
        let travel = offset.length();
        let curvature_squared = curvature * curvature;
        let mut path = Self {
            start,
            offset,
            curvature,
            shape: ZoomPathShape::ZoomOnly,
            length: (end.distance / start.distance).ln() / curvature,
        };
        if travel >= f64::EPSILON * start.distance {
            let squares = end.distance * end.distance - start.distance * start.distance;
            let pan = curvature_squared * curvature_squared * travel * travel;
            let start_slope = (squares + pan) / (2.0 * start.distance * curvature_squared * travel);
            let end_slope = (squares - pan) / (2.0 * end.distance * curvature_squared * travel);
            let start_angle = -start_slope.asinh();
            let end_angle = -end_slope.asinh();
            path.shape = ZoomPathShape::Arc { start_angle, travel };
            path.length = (end_angle - start_angle) / curvature;
        }
        path
    }

    pub fn at(&self, progress: f64) -> ViewPoint {
        let position = progress * self.length;
        match self.shape {
            ZoomPathShape::ZoomOnly => ViewPoint {
                focus: self.start.focus + self.offset * progress,
                distance: self.start.distance * (self.curvature * position).exp(),
            },
            ZoomPathShape::Arc { start_angle, travel } => {
                let angle = self.curvature * position + start_angle;
                let pan_fraction = self.start.distance / (self.curvature * self.curvature * travel)
                    * (start_angle.cosh() * angle.tanh() - start_angle.sinh());
                ViewPoint {
                    focus: self.start.focus + self.offset * pan_fraction,
                    distance: self.start.distance * start_angle.cosh() / angle.cosh(),
                }
            }
        }
    }
}
