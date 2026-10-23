#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IpcTranscript {
    pub enabled: bool,
    pub input: bool,
    pub output: bool,
}

impl IpcTranscript {
    pub fn shows_input(self) -> bool {
        self.enabled && self.input
    }

    pub fn shows_output(self) -> bool {
        self.enabled && self.output
    }
}

impl Default for IpcTranscript {
    fn default() -> Self {
        IpcTranscript {
            enabled: true,
            input: true,
            output: true,
        }
    }
}
