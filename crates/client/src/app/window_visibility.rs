// What the platform reports about the window; Wayland and Windows never report it occluded.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum WindowVisibility {
    Visible,
    Occluded,
}
