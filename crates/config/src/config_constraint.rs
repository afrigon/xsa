// What a key accepts beyond its type, such as a range; described in the schema.
pub trait ConfigConstraint<T> {
    fn check(&self, value: &T) -> anyhow::Result<()>;

    fn description(&self) -> Option<String>;
}
