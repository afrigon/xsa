pub trait ConfigChoice: Copy + PartialEq + 'static {
    const ALL: &'static [Self];

    fn config_name(self) -> &'static str;

    fn names() -> Vec<&'static str> {
        Self::ALL.iter().map(|choice| choice.config_name()).collect()
    }

    fn from_config_name(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|choice| choice.config_name() == name)
    }
}
