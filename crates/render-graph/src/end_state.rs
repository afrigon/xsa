use crate::ImageState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EndState {
    pub image: usize,
    pub level: u32,
    pub state: ImageState,
}
