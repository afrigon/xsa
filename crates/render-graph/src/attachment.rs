use ash::vk;

const CLEAR_STENCIL: u32 = 0;

/// What an attachment holds when rendering begins.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Attachment {
    /// Keeps what earlier passes wrote.
    Load,
    /// Starts from undefined contents the step fully overwrites.
    DontCare,
    ClearColor([f32; 4]),
    ClearDepth(f32),
}

impl Attachment {
    pub(crate) fn preserves_contents(self) -> bool {
        self == Attachment::Load
    }

    pub(crate) fn load_op(self) -> vk::AttachmentLoadOp {
        match self {
            Attachment::Load => vk::AttachmentLoadOp::LOAD,
            Attachment::DontCare => vk::AttachmentLoadOp::DONT_CARE,
            Attachment::ClearColor(_) | Attachment::ClearDepth(_) => vk::AttachmentLoadOp::CLEAR,
        }
    }

    pub(crate) fn clear_value(self) -> vk::ClearValue {
        match self {
            Attachment::ClearColor(color) => vk::ClearValue {
                color: vk::ClearColorValue { float32: color },
            },
            Attachment::ClearDepth(depth) => vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth,
                    stencil: CLEAR_STENCIL,
                },
            },
            Attachment::Load | Attachment::DontCare => vk::ClearValue::default(),
        }
    }
}
