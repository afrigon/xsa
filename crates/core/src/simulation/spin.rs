#[derive(Clone, Copy, Default)]
pub struct Spin {
    pub tidally_locked: bool,
    pub azimuth: f64,
    pub prime_meridian: f64,
}
