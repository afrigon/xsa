use crate::GraphDescription;
use crate::ImageId;
use crate::compiled_graph::CompiledGraph;
use crate::frame_declaration::FrameDeclaration;

pub(crate) struct CompiledEntry {
    pub declaration: FrameDeclaration,
    pub compiled: CompiledGraph,
    pub image_slots: Vec<Option<ImageId>>,
    pub description: GraphDescription,
}
