#[derive(Clone, Copy)]
pub(in crate::renderer) struct ForwardInputs {
    // Linear, alpha last.
    pub clear_color: [f32; 4],
}
