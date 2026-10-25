use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildProfile {
    Debug,
    Release,
}

impl fmt::Display for BuildProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuildProfile::Debug => write!(formatter, "debug"),
            BuildProfile::Release => write!(formatter, "release"),
        }
    }
}
