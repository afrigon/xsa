use crate::compiled_step::CompiledStep;

pub(crate) struct CompiledPass {
    pub declaration: usize,
    pub steps: Vec<CompiledStep>,
}
