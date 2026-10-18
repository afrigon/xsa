use std::fmt;
use std::str::FromStr;

use anyhow::bail;
use glam::IVec3;

/// A face of a body's cube map, in Vulkan's face order and orientation in the body's frame.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CubeFace {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

impl CubeFace {
    /// Every face, in the order of their indices.
    pub const ALL: [CubeFace; 6] = [
        CubeFace::PositiveX,
        CubeFace::NegativeX,
        CubeFace::PositiveY,
        CubeFace::NegativeY,
        CubeFace::PositiveZ,
        CubeFace::NegativeZ,
    ];

    const NAMES: [&str; 6] = ["+x", "-x", "+y", "-y", "+z", "-z"];

    /// The face's index, 0 to 5.
    pub fn index(self) -> usize {
        self as usize
    }

    /// The face whose outward normal is a unit axis.
    pub fn from_normal(normal: IVec3) -> Option<CubeFace> {
        CubeFace::ALL.into_iter().find(|face| face.normal() == normal)
    }

    /// The axis the face looks along, outward.
    pub fn normal(self) -> IVec3 {
        match self {
            CubeFace::PositiveX => IVec3::X,
            CubeFace::NegativeX => IVec3::NEG_X,
            CubeFace::PositiveY => IVec3::Y,
            CubeFace::NegativeY => IVec3::NEG_Y,
            CubeFace::PositiveZ => IVec3::Z,
            CubeFace::NegativeZ => IVec3::NEG_Z,
        }
    }

    /// The axis along which the face's columns run, left to right.
    pub fn column_axis(self) -> IVec3 {
        match self {
            CubeFace::PositiveX => IVec3::NEG_Z,
            CubeFace::NegativeX => IVec3::Z,
            CubeFace::PositiveY | CubeFace::NegativeY | CubeFace::PositiveZ => IVec3::X,
            CubeFace::NegativeZ => IVec3::NEG_X,
        }
    }

    /// The axis along which the face's rows run, top to bottom.
    pub fn row_axis(self) -> IVec3 {
        match self {
            CubeFace::PositiveY => IVec3::Z,
            CubeFace::NegativeY => IVec3::NEG_Z,
            CubeFace::PositiveX | CubeFace::NegativeX | CubeFace::PositiveZ | CubeFace::NegativeZ => IVec3::NEG_Y,
        }
    }
}

impl fmt::Display for CubeFace {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(Self::NAMES[self.index()])
    }
}

/// Reads a face by its name: `+x`, `-x`, `+y`, `-y`, `+z` or `-z`.
impl FromStr for CubeFace {
    type Err = anyhow::Error;

    fn from_str(text: &str) -> anyhow::Result<CubeFace> {
        match CubeFace::ALL.into_iter().find(|face| Self::NAMES[face.index()] == text) {
            Some(face) => Ok(face),
            None => bail!("unknown face {text:?}: expected +x, -x, +y, -y, +z or -z"),
        }
    }
}
