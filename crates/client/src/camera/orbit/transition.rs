use animation::EasedProgress;

use super::view_point::ViewPoint;
use super::zoom_path::ZoomPath;

pub(super) struct Transition {
    pub path: ZoomPath,
    pub end: ViewPoint,
    pub progress: EasedProgress,
}
