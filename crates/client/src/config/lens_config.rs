use xsa_packs::Id;

#[derive(Clone, Debug, PartialEq)]
pub struct LensConfig {
    pub enabled: bool,
    pub profile: Id,
}
