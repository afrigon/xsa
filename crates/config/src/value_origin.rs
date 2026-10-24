use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ValueOrigin {
    Session,
    Saved,
    Default,
}

impl fmt::Display for ValueOrigin {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            ValueOrigin::Session => "session",
            ValueOrigin::Saved => "saved",
            ValueOrigin::Default => "default",
        };

        formatter.write_str(name)
    }
}
