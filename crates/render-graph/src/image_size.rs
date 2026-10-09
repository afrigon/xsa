use ash::vk;

/// The size of a graph image, relative to the output it is rendered for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageSize {
    Output,
    OutputDivided(u32),
    Fixed(vk::Extent2D),
}

impl ImageSize {
    pub(crate) fn extent(self, output: vk::Extent2D) -> vk::Extent2D {
        match self {
            ImageSize::Output => output,
            ImageSize::OutputDivided(divisor) => vk::Extent2D {
                width: (output.width / divisor).max(1),
                height: (output.height / divisor).max(1),
            },
            ImageSize::Fixed(extent) => extent,
        }
    }
}
