use std::fmt;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BodyId {
    pub value: String,
}

impl fmt::Display for BodyId {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(&self.value)
    }
}
