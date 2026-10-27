use crate::renderer::passes::ScatterInputs;

#[derive(Clone, Copy)]
pub(in crate::renderer) struct ForwardInputs {
    // Linear, alpha last.
    pub clear_color: [f32; 4],
    // None for a frame without scatter pages.
    pub scatter: Option<ScatterInputs>,
}
