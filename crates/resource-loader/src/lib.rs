//! Game resource files decoded into ready-to-use data, one module per kind of file, each behind a feature of the same
//! name; `utils` holds the value types they share.

#[cfg(feature = "texture")]
pub mod texture;
pub mod utils;
