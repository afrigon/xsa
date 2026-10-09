use crate::StepDescription;

#[derive(Clone, Debug)]
pub struct PassDescription {
    pub name: &'static str,
    /// Whether the pass was left out because nothing that runs reads what it writes.
    pub culled: bool,
    pub steps: Vec<StepDescription>,
}
