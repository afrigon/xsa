use std::path::PathBuf;

// A body's height map: a KTX2 cube map of 16-bit values spanning `minimum` to `maximum`, meters relative to the body's
// radius.
pub struct HeightMapDefinition {
    pub path: PathBuf,
    pub minimum: f64,
    pub maximum: f64,
}
