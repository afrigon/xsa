use crate::{PassContext, PassError, RenderPass};

// RenderPass without its associated types, so passes of different types share one list.
pub(crate) trait RecordedPass<Context> {
    fn record(&mut self, context: &Context, pass: &PassContext) -> Result<(), PassError>;
}

impl<Context, Pass: RenderPass<Context>> RecordedPass<Context> for Pass {
    fn record(&mut self, context: &Context, pass: &PassContext) -> Result<(), PassError> {
        RenderPass::record(self, context, pass)
    }
}
