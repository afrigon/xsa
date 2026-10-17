#[cfg(target_os = "linux")]
mod linux;
#[cfg(windows)]
mod windows;

#[cfg(target_os = "linux")]
pub(crate) use linux::AvailableMemory;
#[cfg(windows)]
pub(crate) use windows::AvailableMemory;
