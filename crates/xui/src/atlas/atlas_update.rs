use super::AtlasRegion;

// One byte of coverage per pixel, rows packed without padding.
#[derive(Clone, Debug)]
pub struct AtlasUpdate {
    pub region: AtlasRegion,
    pub pixels: Vec<u8>,
}
