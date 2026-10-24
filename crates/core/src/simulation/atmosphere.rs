#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Atmosphere {
    // Radians the star sinks below the horizon before its light leaves the sky; without it, the sky goes dark at
    // sunset.
    pub twilight_depression: Option<f64>,
}
