use crate::ConfigConstraint;

pub(crate) struct Unconstrained;

impl<T> ConfigConstraint<T> for Unconstrained {
    fn check(&self, _value: &T) -> anyhow::Result<()> {
        Ok(())
    }

    fn description(&self) -> Option<String> {
        None
    }
}
