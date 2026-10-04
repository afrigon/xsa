use super::motion::Motion;

pub(super) enum Offset {
    Origin,
    Orbit(Motion),
    Share { motion: Motion, factor: f64 },
}

impl Offset {
    pub fn motion(&self) -> Option<Motion> {
        match self {
            Offset::Orbit(motion) => Some(*motion),
            Offset::Origin | Offset::Share { .. } => None,
        }
    }
}
