use xsa_config::{ConfigSection, ConfigVisitor};

use super::{AntialiasingKind, BloomConfig, FadeConfig, LensConfig, WhiteBalanceConfig};
use crate::renderer::Tonemapper;

#[derive(Clone, Debug, PartialEq)]
pub struct VideoConfig {
    pub tonemapper: Option<Tonemapper>,
    pub stars: bool,
    pub antialiasing: Option<AntialiasingKind>,
    pub bloom: BloomConfig,
    pub lens: LensConfig,
    pub white_balance: WhiteBalanceConfig,
    pub fade: FadeConfig,
}

impl ConfigSection for VideoConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.value("tonemapper", &mut self.tonemapper);
        visitor.value("stars", &mut self.stars);
        visitor.value("antialiasing", &mut self.antialiasing);
        visitor.section("bloom", &mut self.bloom);
        visitor.section("lens", &mut self.lens);
        visitor.section("white-balance", &mut self.white_balance);
        visitor.section("fade", &mut self.fade);
    }
}
