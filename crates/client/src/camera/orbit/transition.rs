use super::view_point::ViewPoint;
use super::zoom_path::ZoomPath;

pub(super) struct Transition {
    pub path: ZoomPath,
    pub end: ViewPoint,
    pub elapsed: f64,
    pub duration: f64,
}

impl Transition {
    pub fn eased_progress(&self) -> f64 {
        let progress = (self.elapsed / self.duration).min(1.0);

        progress * progress * (3.0 - 2.0 * progress)
    }
}
