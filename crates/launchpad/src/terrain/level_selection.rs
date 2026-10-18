use std::fmt;

/// Which levels of a `.terrain` file an extract holds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LevelSelection {
    /// Every level, as a mip chain, finest first.
    Every,
    One(u32),
}

impl fmt::Display for LevelSelection {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LevelSelection::Every => formatter.write_str("every level"),
            LevelSelection::One(level) => write!(formatter, "level {level}"),
        }
    }
}
