use super::Axis;
use crate::{Alignment, DrawList, Rect, Size, SizeProposal, Subview, View, ViewContext};

pub(crate) const DEFAULT_SPACING: f32 = 8.0;

// SwiftUI's stack layout: the least flexible subviews are sized first, each offered an equal share of the space
// left, so flexible ones (a Spacer, a wrapping Text) absorb what remains.
#[derive(Clone, Copy, Debug)]
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

    pub fn size_of_content(
        &self,
        content: &impl View,
        proposal: SizeProposal,
        context: &mut ViewContext,
    ) -> anyhow::Result<Size> {
        let mut subviews = Vec::new();
        content.collect_subviews(&mut subviews);
        let sizes = self.subview_sizes(&subviews, proposal, context)?;

        Ok(self.total(&sizes))
    }

    pub fn place_content(
        &self,
        content: &impl View,
        bounds: Rect,
        context: &mut ViewContext,
        draw_list: &mut DrawList,
    ) -> anyhow::Result<()> {
        let mut subviews = Vec::new();
        content.collect_subviews(&mut subviews);
        let sizes = self.subview_sizes(&subviews, SizeProposal::from(bounds.size), context)?;
        let axis = self.axis;
        let mut context = context.with_stack_axis(Some(axis));
        let mut main = axis.main_coordinate(bounds.origin);

        for (subview, size) in subviews.iter().zip(sizes) {
            let cross = axis.cross_coordinate(bounds.origin)
                + axis.cross_offset(self.alignment, axis.cross(bounds.size), axis.cross(size));
            let origin = axis.point(main, cross);
            subview.arrange(Rect { origin, size }, &mut context, draw_list)?;
            main += axis.main(size) + self.spacing;
        }

        Ok(())
    }

    fn subview_sizes(
        &self,
        subviews: &[&dyn Subview],
        proposal: SizeProposal,
        context: &mut ViewContext,
    ) -> anyhow::Result<Vec<Size>> {
        let axis = self.axis;
        let cross = axis.cross_proposal(proposal);
        let mut context = context.with_stack_axis(Some(axis));
        let Some(length) = axis.main_proposal(proposal) else {
            return subviews
                .iter()
                .map(|subview| subview.measure(axis.proposal(None, cross), &mut context))
                .collect();
        };
        let mut flexibilities = Vec::with_capacity(subviews.len());

        for subview in subviews {
            let smallest = subview.measure(axis.proposal(Some(0.0), cross), &mut context)?;
            let largest = subview.measure(axis.proposal(Some(f32::INFINITY), cross), &mut context)?;
            flexibilities.push(axis.main(largest) - axis.main(smallest));
        }

        let mut order: Vec<usize> = (0..subviews.len()).collect();
        order.sort_by(|first, second| flexibilities[*first].total_cmp(&flexibilities[*second]));
        let mut remaining = length - self.spacing_total(subviews.len());
        let mut sizes = vec![Size::default(); subviews.len()];

        for (sized, index) in order.into_iter().enumerate() {
            let share = (remaining / (subviews.len() - sized) as f32).max(0.0);
            let size = subviews[index].measure(axis.proposal(Some(share), cross), &mut context)?;
            remaining -= axis.main(size);
            sizes[index] = size;
        }

        Ok(sizes)
    }

    fn total(&self, sizes: &[Size]) -> Size {
        let main = sizes.iter().map(|size| self.axis.main(*size)).sum::<f32>() + self.spacing_total(sizes.len());
        let cross = sizes.iter().map(|size| self.axis.cross(*size)).fold(0.0, f32::max);

        self.axis.size(main, cross)
    }

    fn spacing_total(&self, count: usize) -> f32 {
        self.spacing * count.saturating_sub(1) as f32
    }
}
