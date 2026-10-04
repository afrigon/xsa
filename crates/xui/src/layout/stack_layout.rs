use super::Axis;
use crate::{Alignment, DrawList, LayoutContext, Node, NodeLayout, Rect, Size, SizeProposal};

pub(crate) const DEFAULT_SPACING: f32 = 8.0;

// SwiftUI's stack layout: the least flexible children are sized first, each offered an equal share of the space
// left, so flexible ones (a Spacer, a wrapping Text) absorb what remains.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct StackLayout {
    pub axis: Axis,
    pub alignment: Alignment,
    pub spacing: f32,
}

impl StackLayout {
    // How a bare list of views lays out, like SwiftUI's implicit vertical stacking.
    pub fn list() -> StackLayout {
        StackLayout {
            axis: Axis::Vertical,
            alignment: Alignment::CENTER,
            spacing: DEFAULT_SPACING,
        }
    }

    fn child_sizes(
        &self,
        children: &mut [Node],
        proposal: SizeProposal,
        context: &mut LayoutContext,
    ) -> anyhow::Result<Vec<Size>> {
        let axis = self.axis;
        let cross = axis.cross_proposal(proposal);
        let mut context = context.with_stack_axis(Some(axis));
        let Some(length) = axis.main_proposal(proposal) else {
            return children
                .iter_mut()
                .map(|child| child.size_that_fits(axis.proposal(None, cross), &mut context))
                .collect();
        };
        let mut flexibilities = Vec::with_capacity(children.len());

        for child in children.iter_mut() {
            let smallest = child.size_that_fits(axis.proposal(Some(0.0), cross), &mut context)?;
            let largest = child.size_that_fits(axis.proposal(Some(f32::INFINITY), cross), &mut context)?;
            flexibilities.push(axis.main(largest) - axis.main(smallest));
        }

        let mut order: Vec<usize> = (0..children.len()).collect();
        order.sort_by(|first, second| flexibilities[*first].total_cmp(&flexibilities[*second]));
        let mut remaining = length - self.spacing_total(children.len());
        let mut sizes = vec![Size::default(); children.len()];

        for (sized, index) in order.into_iter().enumerate() {
            let share = (remaining / (children.len() - sized) as f32).max(0.0);
            let size = children[index].size_that_fits(axis.proposal(Some(share), cross), &mut context)?;
            remaining -= axis.main(size);
            sizes[index] = size;
        }

        Ok(sizes)
    }

    fn spacing_total(&self, count: usize) -> f32 {
        self.spacing * count.saturating_sub(1) as f32
    }
}

impl NodeLayout for StackLayout {
    fn size_that_fits(
        &mut self,
        proposal: SizeProposal,
        children: &mut [Node],
        context: &mut LayoutContext,
    ) -> anyhow::Result<Size> {
        let sizes = self.child_sizes(children, proposal, context)?;
        let main = sizes.iter().map(|size| self.axis.main(*size)).sum::<f32>() + self.spacing_total(sizes.len());
        let cross = sizes.iter().map(|size| self.axis.cross(*size)).fold(0.0, f32::max);

        Ok(self.axis.size(main, cross))
    }

    fn place(
        &mut self,
        bounds: Rect,
        children: &mut [Node],
        context: &mut LayoutContext,
        draw_list: &mut DrawList,
    ) -> anyhow::Result<()> {
        let sizes = self.child_sizes(children, SizeProposal::from(bounds.size), context)?;
        let axis = self.axis;
        let mut context = context.with_stack_axis(Some(axis));
        let mut main = axis.main_coordinate(bounds.origin);

        for (child, size) in children.iter_mut().zip(sizes) {
            let cross = axis.cross_coordinate(bounds.origin)
                + axis.cross_offset(self.alignment, axis.cross(bounds.size), axis.cross(size));
            let origin = axis.point(main, cross);
            child.place(Rect { origin, size }, &mut context, draw_list)?;
            main += axis.main(size) + self.spacing;
        }

        Ok(())
    }
}
