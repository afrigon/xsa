use crate::renderer::static_buffer::StaticBuffer;

// A page's instances in GPU memory, drawn once their upload completes; a page without instances has no buffer.
pub(in crate::renderer) struct ScatterPage {
    pub buffer: Option<StaticBuffer>,
    pub uploaded: bool,
    pub instance_count: u32,
    // By the model's place in its scatter.
    pub model_counts: Vec<u32>,
}

impl ScatterPage {
    pub fn is_drawn(&self) -> bool {
        self.uploaded && self.instance_count > 0
    }
}
