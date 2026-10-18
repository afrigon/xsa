use crate::shell::Shell;

pub struct LaunchpadContext {
    shell: Shell,
}

impl LaunchpadContext {
    pub fn new(shell: Shell) -> LaunchpadContext {
        LaunchpadContext { shell }
    }

    pub fn shell(&mut self) -> &mut Shell {
        &mut self.shell
    }
}
