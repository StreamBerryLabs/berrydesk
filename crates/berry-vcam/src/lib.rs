//! Virtual camera abstraction.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VcamError {
    #[error("device not found: {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("unsupported platform")]
    Unsupported,
    #[error("{0}")]
    Other(String),
}

/// A running virtual camera sink.
pub trait VirtualCamera: Send {
    fn name(&self) -> &str;
    fn write_frame(
        &mut self,
        data: &[u8],
        width: u32,
        height: u32,
        ts_us: u64,
    ) -> Result<(), VcamError>;
    fn stop(&mut self) -> Result<(), VcamError>;
}

pub fn default_device_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        Some(PathBuf::from("/dev/video10"))
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}
