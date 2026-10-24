//! Frames: the values a connection carries in each direction.

mod client_frame;
mod server_frame;

pub use client_frame::ClientFrame;
pub use server_frame::ServerFrame;
