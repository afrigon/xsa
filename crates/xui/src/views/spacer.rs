use crate::{DrawList, Environment, Never, Rect, Size, SizeProposal, View, ViewContext};

// Takes the space offered along its stack's axis, so the views around it are pushed apart. Outside a stack it
// takes the space offered on both axes.
#[derive(Default)]
pub struct Spacer {
    min_length: f32,
}

impl Spacer {
    pub fn new() -> Spacer {
        Spacer::default()
    }

    pub fn min_length(self, min_length: f32) -> Spacer {
        Spacer { min_length }
    }
}

impl View for Spacer {
    fn body(&self, _environment: &Environment) -> impl View {
        Never::primitive_body()
    }

    fn size_that_fits(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size> {
        let Some(axis) = context.stack_axis else {
            return Ok(Size {
                width: proposal.width.unwrap_or(self.min_length).max(self.min_length),
                height: proposal.height.unwrap_or(self.min_length).max(self.min_length),
            });
        };
        let length = axis
            .main_proposal(proposal)
            .unwrap_or(self.min_length)
            .max(self.min_length);

        Ok(axis.size(length, 0.0))
    }

    fn place(&self, _bounds: Rect, _context: &mut ViewContext, _draw_list: &mut DrawList) -> anyhow::Result<()> {
        Ok(())
    }
}
