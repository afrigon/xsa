use std::collections::VecDeque;

use crate::TransferPriority;
use crate::transfer_queue::{Piece, TransferCompletion, TransferWork};

// A request waiting for, or partway through, its copies.
pub(crate) struct TransferRequest {
    pub priority: TransferPriority,
    pub work: TransferWork,
    pub pieces: VecDeque<Piece>,
    // Whether a piece was recorded, so an owned image has left its undefined layout.
    pub started: bool,
    pub completion: Box<dyn TransferCompletion>,
}
