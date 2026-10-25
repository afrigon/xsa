/// One encoded frame: H.265 NAL units in Annex B form, each after a start code.
pub struct EncodedFrame {
    pub bytes: Vec<u8>,
    /// Whether the frame decodes on its own, so a player can start or seek there.
    pub keyframe: bool,
}
