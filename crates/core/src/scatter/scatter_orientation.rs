// Which way a placed model's up axis points.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScatterOrientation {
    // Along the ground's normal, as a rock lies.
    Ground,
    // Away from the body's center, as a tree grows.
    Up,
}
