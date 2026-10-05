use super::AntialiasingKind;

#[derive(Clone, Debug, PartialEq)]
pub struct AntialiasingConfig {
    pub kind: Option<AntialiasingKind>,
}
