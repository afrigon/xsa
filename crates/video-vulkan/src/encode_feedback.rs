// What a feedback query reports for one encode, in the order its flags are requested, followed by the status.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct EncodeFeedback {
    pub offset: u32,
    pub bytes: u32,
    pub status: i32,
}
