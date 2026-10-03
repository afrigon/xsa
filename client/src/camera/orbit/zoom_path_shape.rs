pub(super) enum ZoomPathShape {
    ZoomOnly,
    Arc { start_angle: f64, travel: f64 },
}
