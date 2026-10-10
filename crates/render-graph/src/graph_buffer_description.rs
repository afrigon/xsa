use ash::vk;

/// A buffer the graph allocates, in GPU memory. Shaders reach it through its device address; transfer usages follow
/// from how passes declare they use it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GraphBufferDescription {
    pub name: &'static str,
    pub size: vk::DeviceSize,
}
