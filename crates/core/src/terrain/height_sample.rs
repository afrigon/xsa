// A height in meters relative to the body's radius, read on a page of `level`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct HeightSample {
    pub height: f64,
    pub level: u32,
}
