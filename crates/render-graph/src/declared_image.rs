use ash::vk;

use crate::{GraphImageDescription, HistoryId, ImageState};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DeclaredImage {
    Transient(GraphImageDescription),
    History {
        id: HistoryId,
        description: GraphImageDescription,
        previous: bool,
    },
    Imported {
        import: usize,
        name: &'static str,
        aspect: vk::ImageAspectFlags,
        initial: ImageState,
        final_state: ImageState,
    },
}

impl DeclaredImage {
    pub fn name(&self) -> &'static str {
        match self {
            DeclaredImage::Transient(description) | DeclaredImage::History { description, .. } => description.name,
            DeclaredImage::Imported { name, .. } => name,
        }
    }

    pub fn level_count(&self) -> u32 {
        match self {
            DeclaredImage::Transient(description) | DeclaredImage::History { description, .. } => {
                description.mip_levels
            }
            DeclaredImage::Imported { .. } => 1,
        }
    }

    pub fn persists(&self) -> bool {
        !matches!(self, DeclaredImage::Transient(_))
    }
}
