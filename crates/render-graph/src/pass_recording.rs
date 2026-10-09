use crate::recorded_pass::RecordedPass;
use crate::{PassContext, PassError, RenderPass};

pub(crate) struct PassRecording<'passes, Pass, Resources> {
    pub pass: &'passes mut Pass,
    pub resources: Resources,
}

impl<Context, Pass: RenderPass<Context>> RecordedPass<Context> for PassRecording<'_, Pass, Pass::Resources> {
    fn record(&mut self, context: &Context, pass: &PassContext) -> Result<(), PassError> {
        self.pass.record(context, pass, &self.resources)
    }
}
