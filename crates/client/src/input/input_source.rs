#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InputSource {
    Raw,
    Command,
}

impl InputSource {
    pub const ALL: [InputSource; 2] = [InputSource::Raw, InputSource::Command];
}
