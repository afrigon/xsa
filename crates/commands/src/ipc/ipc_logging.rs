#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IpcLogging {
    pub enabled: bool,
    pub input: bool,
    pub output: bool,
}

impl IpcLogging {
    pub fn logs_input(self) -> bool {
        self.enabled && self.input
    }

    pub fn logs_output(self) -> bool {
        self.enabled && self.output
    }
}

impl Default for IpcLogging {
    fn default() -> Self {
        IpcLogging {
            enabled: true,
            input: true,
            output: true,
        }
    }
}
