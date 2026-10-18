//! Game resource files decoded into ready-to-use data, one module per kind of file, each behind a feature of the same
//! name.

#[cfg(feature = "terrain")]
pub mod terrain;
#[cfg(feature = "texture")]
pub mod texture;
