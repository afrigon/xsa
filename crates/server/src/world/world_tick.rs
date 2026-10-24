use xsa_units::TimeRate;

// What one tick of the world did to time.
pub(crate) struct WorldTick {
    // The rate time ran at over the tick.
    pub rate: TimeRate,
    // How time runs changed during the tick: a warp reached its end.
    pub flow_changed: bool,
}
