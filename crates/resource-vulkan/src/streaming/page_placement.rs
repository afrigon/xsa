use ash::vk;

/// Where shaders read a page from: `page` is the requested page or its nearest resident coarser page, at texel
/// `offset` in the texture at index `slot` of the bindless textures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PagePlacement<K> {
    pub page: K,
    pub slot: u32,
    pub offset: vk::Offset2D,
}
