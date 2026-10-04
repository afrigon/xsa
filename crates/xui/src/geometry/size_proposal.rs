use super::{EdgeInsets, Size};

// A dimension left unspecified asks a view for its ideal size along it.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct SizeProposal {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl SizeProposal {
    pub fn inset(self, insets: EdgeInsets) -> SizeProposal {
        SizeProposal {
            width: self.width.map(|width| (width - insets.horizontal()).max(0.0)),
            height: self.height.map(|height| (height - insets.vertical()).max(0.0)),
        }
    }
}

impl From<Size> for SizeProposal {
    fn from(size: Size) -> SizeProposal {
        SizeProposal {
            width: Some(size.width),
            height: Some(size.height),
        }
    }
}
