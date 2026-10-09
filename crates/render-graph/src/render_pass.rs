use crate::{PassContext, PassDeclaration, PassError};

/// A pass of the graph. `Context` is whatever the host hands every pass while recording; the graph never looks
/// inside it.
pub trait RenderPass<Context> {
    const NAME: &'static str;

    /// The handles the pass needs from earlier passes, such as the images it reads.
    type Inputs;
    /// The handles the pass records with this frame. `GraphBuilder::add` also returns them, to wire later passes.
    type Resources: Copy;

    /// Declares what the pass creates and uses this frame. Called once per frame.
    fn declare(&self, pass: &mut PassDeclaration, inputs: Self::Inputs) -> Self::Resources;

    /// Records the pass's commands; the graph records the barriers before each step.
    fn record(&mut self, context: &Context, pass: &PassContext, resources: &Self::Resources) -> Result<(), PassError>;
}
