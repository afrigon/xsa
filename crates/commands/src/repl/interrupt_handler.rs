use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use rustyline::{Cmd, ConditionalEventHandler, Event, EventContext, RepeatCount};

// Ctrl-C clears a line being typed, like readline; on an empty line it exits, like the signal it replaces.
pub(super) struct InterruptHandler {
    pub empty_line: Arc<AtomicBool>,
}

impl ConditionalEventHandler for InterruptHandler {
    fn handle(&self, _event: &Event, _count: RepeatCount, _positive: bool, context: &EventContext) -> Option<Cmd> {
        self.empty_line.store(context.line().is_empty(), Ordering::Relaxed);

        Some(Cmd::Interrupt)
    }
}
