use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use ash::vk;

use crate::streaming::{CellAddress, CellContent};

// What a set holds of one page.
pub(crate) enum PageState {
    // Nothing: never read, or its bytes were evicted.
    Unloaded,
    // A worker reads it; setting `cancel` makes the worker skip it if it has not started. The worker reports back
    // either way.
    Reading {
        cancel: Arc<AtomicBool>,
    },
    // Its bytes are in RAM, in no cell.
    Cached,
    // Its bytes are copying into `cell`.
    Uploading {
        cell: CellAddress,
    },
    // In `cell`, read by shaders through the chunk's bindless index `slot` at texel `offset`.
    Resident {
        cell: CellAddress,
        slot: u32,
        offset: vk::Offset2D,
    },
    // Resident, and copying into `destination` because `cell`'s chunk is being released. Shaders read `cell` until
    // the copy completes.
    Moving {
        cell: CellAddress,
        slot: u32,
        offset: vk::Offset2D,
        destination: CellAddress,
    },
    // The source has no data for it.
    Absent,
    // Reading it failed; it is not read again.
    Failed,
}

impl PageState {
    // The cell shaders read it from, or it is copying into.
    pub fn cell(&self) -> Option<CellAddress> {
        match self {
            PageState::Uploading { cell } | PageState::Resident { cell, .. } | PageState::Moving { cell, .. } => {
                Some(*cell)
            }
            _ => None,
        }
    }

    // Every cell it holds: a moving page holds two.
    pub fn cells(&self) -> Vec<CellAddress> {
        match self {
            PageState::Moving { cell, destination, .. } => vec![*cell, *destination],
            _ => self.cell().into_iter().collect(),
        }
    }

    pub fn content(&self) -> Option<CellContent> {
        match self {
            PageState::Uploading { .. } => Some(CellContent::Uploading),
            PageState::Resident { .. } => Some(CellContent::Resident),
            PageState::Moving { .. } => Some(CellContent::Moving),
            _ => None,
        }
    }
}
