use super::CubeTexel;

// Where a texel's value comes from: a stored texel, or at a cube corner, where three faces meet and no diagonal
// neighbour exists, the average of three.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TexelSource {
    Stored(CubeTexel),
    Corner([CubeTexel; 3]),
}
