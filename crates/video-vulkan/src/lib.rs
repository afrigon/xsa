//! H.265 video encoding on the GPU's hardware encoder through Vulkan Video.
//!
//! A `ColorConversion` records the conversion of a rendered frame into the encoder's YUV input on the host's command
//! buffer; a `VideoEncoder` encodes it on the encode queue and hands back each frame's bitstream; an `Mp4Writer` wraps
//! the bitstream in an MP4 file. Every GPU object follows `memory-vulkan`'s lifetime rules.

mod color_conversion;
mod conversion_constants;
mod conversion_source;
mod encode_feedback;
mod encode_session;
mod encoded_frame;
mod encoder_capabilities;
mod h265_profile;
mod mp4_box;
mod mp4_writer;
mod nal_units;
mod picture_geometry;
mod picture_syntax;
mod reference_picture;
mod video_encoder;
mod video_error;
mod video_session;
mod video_session_parameters;

pub use color_conversion::ColorConversion;
pub use conversion_source::ConversionSource;
pub use encoded_frame::EncodedFrame;
pub use mp4_writer::Mp4Writer;
pub use video_encoder::VideoEncoder;
pub use video_error::VideoError;
