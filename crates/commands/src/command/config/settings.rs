use std::fmt;

// A count of settings, as replies word it: `1 setting`, `3 settings`.
pub(super) struct Settings {
    pub count: usize,
}

impl fmt::Display for Settings {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.count {
            1 => write!(formatter, "1 setting"),
            count => write!(formatter, "{count} settings"),
        }
    }
}
