// How the camera moves from the controls it leaves to the ones it takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraSwitch {
    Cut,
    Smooth,
}
