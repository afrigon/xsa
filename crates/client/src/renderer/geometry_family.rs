// How a program's vertices are placed: the vertex input its pipelines take and the draw record it reads.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GeometryFamily {
    Terrain,
    Mesh,
    // A fullscreen triangle, drawn by its own pass.
    Sky,
}
