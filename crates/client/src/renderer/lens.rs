use resource_vulkan::ResourceHandle;
use xsa_packs::LensDefinition;

use super::gpu_context::GpuContext;
use super::lens_flare::LensFlare;
use crate::resource_kinds::{LensGhost, LensStarburst};

// A lens and its textures. Its flare is derived once every texture is ready, and drawn once uploaded; a missing or
// failed texture leaves it undrawn.
pub(in crate::renderer) struct Lens {
    definition: LensDefinition,
    starburst: ResourceHandle<LensStarburst>,
    ghosts: Vec<ResourceHandle<LensGhost>>,
    flare: Option<LensFlare>,
}

impl Lens {
    pub fn new(
        definition: LensDefinition,
        starburst: ResourceHandle<LensStarburst>,
        ghosts: Vec<ResourceHandle<LensGhost>>,
    ) -> Lens {
        Lens {
            definition,
            starburst,
            ghosts,
            flare: None,
        }
    }

    pub fn refresh(&mut self, gpu: &GpuContext) -> anyhow::Result<()> {
        if self.flare.is_none()
            && let Some(starburst) = self.starburst.get()
            && let Some(ghosts) = self.ghosts.iter().map(ResourceHandle::get).collect::<Option<Vec<_>>>()
        {
            let mut flare = LensFlare::new(&self.definition, starburst, &ghosts);
            flare.upload(gpu)?;
            self.flare = Some(flare);
        }

        if let Some(flare) = &mut self.flare {
            flare.poll_ghosts()?;
        }

        Ok(())
    }

    // None until its flare can be drawn.
    pub fn flare(&self) -> Option<&LensFlare> {
        self.flare.as_ref().filter(|flare| flare.is_uploaded())
    }
}
