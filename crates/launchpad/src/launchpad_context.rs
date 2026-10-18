use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::shell::Shell;

pub struct LaunchpadContext {
    shell: Arc<Mutex<Shell>>,
}

impl LaunchpadContext {
    pub fn new(shell: Shell) -> LaunchpadContext {
        LaunchpadContext {
            shell: Arc::new(Mutex::new(shell)),
        }
    }

    pub fn shell(&self) -> MutexGuard<'_, Shell> {
        self.shell.lock().unwrap_or_else(PoisonError::into_inner)
    }

    #[expect(dead_code, reason = "for the commands that print from spawned tasks")]
    pub fn shell_handle(&self) -> Arc<Mutex<Shell>> {
        self.shell.clone()
    }
}
