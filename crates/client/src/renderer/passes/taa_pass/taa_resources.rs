use render_graph_vulkan::HistoryImage;

use crate::renderer::passes::ForwardResources;

#[derive(Clone, Copy)]
pub(in crate::renderer) struct TaaResources {
    pub forward: ForwardResources,
    pub history: HistoryImage,
}
