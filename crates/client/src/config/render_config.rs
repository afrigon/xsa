use xsa_config::{ConfigSection, ConfigVisitor};

use crate::renderer::{DebugView, SurfaceDebugView, TerrainDebugView};

#[derive(Clone, Debug, PartialEq)]
pub struct RenderConfig {
    // None shades every draw with its program.
    pub shader_override: Option<DebugView>,
    // None leaves terrain draws to the generic override, or to their program.
    pub terrain_shader_override: Option<TerrainDebugView>,
    // None leaves surfaced terrain draws to the terrain and generic overrides, or to their program.
    pub surface_shader_override: Option<SurfaceDebugView>,
}

impl ConfigSection for RenderConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.group("shader", |visitor| {
            visitor.value("override", &mut self.shader_override);
            visitor.group("terrain", |visitor| {
                visitor.value("override", &mut self.terrain_shader_override)
            });
            visitor.group("surface", |visitor| {
                visitor.value("override", &mut self.surface_shader_override)
            });
        });
    }
}
