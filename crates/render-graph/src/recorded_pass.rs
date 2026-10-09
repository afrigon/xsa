use crate::{PassContext, PassError};

// A pass and the resources it declared this frame, without their types, so passes of different types share one list.
pub(crate) trait RecordedPass<Context> {
    fn record(&mut self, context: &Context, pass: &PassContext) -> Result<(), PassError>;
}
