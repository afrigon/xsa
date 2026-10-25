use ash::vk::native::StdVideoH265PictureType;

// A picture in the decoded picture buffer: its slot there, its order in the stream and its type.
#[derive(Clone, Copy)]
pub(crate) struct ReferencePicture {
    pub slot: u32,
    pub order: i32,
    pub picture_type: StdVideoH265PictureType,
}
