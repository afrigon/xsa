// How a placed model takes part in the simulation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScatterPhysics {
    // Seen, never touched.
    None,
    // Collides, never moves.
    Static,
    // Collides and moves when struck.
    Dynamic,
}
