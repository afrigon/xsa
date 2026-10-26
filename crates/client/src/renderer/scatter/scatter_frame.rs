use super::scatter_list::ScatterList;
use super::scatter_mesh_draw::ScatterMeshDraw;
use crate::renderer::gpu_data::{ScatterDrawData, ScatterModelData, ScatterPageData, ScatterUseData};

// What a frame draws of its scatters: the pages it culls, their uses and models, the instance list of each model drawn
// and the submesh draws reading them, in step with `draws`.
#[derive(Default)]
pub(in crate::renderer) struct ScatterFrame {
    pub pages: Vec<ScatterPageData>,
    pub uses: Vec<ScatterUseData>,
    pub models: Vec<ScatterModelData>,
    pub lists: Vec<ScatterList>,
    pub draws: Vec<ScatterDrawData>,
    pub mesh_draws: Vec<ScatterMeshDraw>,
    pub instance_count: u32,
}

impl ScatterFrame {
    pub fn clear(&mut self) {
        self.pages.clear();
        self.uses.clear();
        self.models.clear();
        self.lists.clear();
        self.draws.clear();
        self.mesh_draws.clear();
        self.instance_count = 0;
    }
}
