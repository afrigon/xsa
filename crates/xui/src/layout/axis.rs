use crate::{Alignment, Point, Size, SizeProposal};

// The direction a stack runs in. "Main" is along it, "cross" is across it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    pub fn main(self, size: Size) -> f32 {
        match self {
            Axis::Horizontal => size.width,
            Axis::Vertical => size.height,
        }
    }

    pub fn cross(self, size: Size) -> f32 {
        match self {
            Axis::Horizontal => size.height,
            Axis::Vertical => size.width,
        }
    }

    pub fn size(self, main: f32, cross: f32) -> Size {
        match self {
            Axis::Horizontal => Size {
                width: main,
                height: cross,
            },
            Axis::Vertical => Size {
                width: cross,
                height: main,
            },
        }
    }

    pub fn main_proposal(self, proposal: SizeProposal) -> Option<f32> {
        match self {
            Axis::Horizontal => proposal.width,
            Axis::Vertical => proposal.height,
        }
    }

    pub fn cross_proposal(self, proposal: SizeProposal) -> Option<f32> {
        match self {
            Axis::Horizontal => proposal.height,
            Axis::Vertical => proposal.width,
        }
    }

    pub fn proposal(self, main: Option<f32>, cross: Option<f32>) -> SizeProposal {
        match self {
            Axis::Horizontal => SizeProposal {
                width: main,
                height: cross,
            },
            Axis::Vertical => SizeProposal {
                width: cross,
                height: main,
            },
        }
    }

    pub fn main_coordinate(self, point: Point) -> f32 {
        match self {
            Axis::Horizontal => point.x,
            Axis::Vertical => point.y,
        }
    }

    pub fn cross_coordinate(self, point: Point) -> f32 {
        match self {
            Axis::Horizontal => point.y,
            Axis::Vertical => point.x,
        }
    }

    pub fn point(self, main: f32, cross: f32) -> Point {
        match self {
            Axis::Horizontal => Point { x: main, y: cross },
            Axis::Vertical => Point { x: cross, y: main },
        }
    }

    pub fn cross_offset(self, alignment: Alignment, available: f32, used: f32) -> f32 {
        match self {
            Axis::Horizontal => alignment.vertical.offset(available, used),
            Axis::Vertical => alignment.horizontal.offset(available, used),
        }
    }
}
