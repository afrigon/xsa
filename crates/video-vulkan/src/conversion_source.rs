use ash::vk;

/// The rendered frame a `ColorConversion` reads, sampled through `view` in `layout`.
pub struct ConversionSource {
    pub view: vk::ImageView,
    pub format: vk::Format,
    pub layout: vk::ImageLayout,
    pub extent: vk::Extent2D,
}

impl ConversionSource {
    // Reading an sRGB view decodes to linear light, which the conversion encodes back.
    pub(crate) fn loads_linear(&self) -> bool {
        matches!(
            self.format,
            vk::Format::B8G8R8A8_SRGB | vk::Format::R8G8B8A8_SRGB | vk::Format::A8B8G8R8_SRGB_PACK32
        )
    }
}
