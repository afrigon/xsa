/// Where shaders read a page from: `page` is the requested page or its nearest resident coarser page, in `layer` of
/// the texture array at index `slot` of the bindless texture arrays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PagePlacement<K> {
    pub page: K,
    pub slot: u32,
    pub layer: u32,
}
