use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Next,
    Previous,
    Parent,
    Body { id: String },
}

impl FromStr for Target {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "next" => Ok(Target::Next),
            "previous" => Ok(Target::Previous),
            "parent" => Ok(Target::Parent),
            "" => Err("a target is a body id, next, previous or parent".to_string()),
            id => Ok(Target::Body { id: id.to_string() }),
        }
    }
}
