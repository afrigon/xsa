use render_graph::HistoryImage;

use crate::renderer::passes::ForwardResources;

#[derive(Clone, Copy)]
pub(in crate::renderer) struct TaaResources {
    pub forward: ForwardResources,
    pub history: HistoryImage,
}
