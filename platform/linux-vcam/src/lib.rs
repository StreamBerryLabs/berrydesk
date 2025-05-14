//! Linux virtual camera helpers (v4l2loopback).
//!
//! The actual frame writing is performed by ffmpeg targeting the loopback device.
//! This crate provides device discovery and setup guidance.

use berry_core::Result;
use std::path::PathBuf;

pub fn find_loopback_device() -> Option<PathBuf> {
    for i in 0..64 {
        let sys = format!("/sys/class/video4linux/video{}/name", i);
        if let Ok(name) = std::fs::read_to_string(&sys)
            && name.to_lowercase().contains("loopback")
        {
            return Some(PathBuf::from(format!("/dev/video{}", i)));
        }
    }
    None
}

pub fn setup_instructions() -> &'static str {
    "Install v4l2loopback:\n\
     sudo pacman -S v4l2loopback-dkms  (Arch)\n\
     sudo apt install v4l2loopback-dkms (Debian/Ubuntu)\n\
     sudo modprobe v4l2loopback devices=1 video_nr=10 card_label='BerryDesk Camera' exclusive_caps=1\n\
     Add to /etc/modules-load.d/v4l2loopback.conf for persistence.\n"
}

pub fn ensure_device() -> Result<PathBuf> {
    if let Some(d) = find_loopback_device() {
        Ok(d)
    } else {
        Err(berry_core::BerryError::NotFound(
            "no v4l2loopback device found; run: sudo modprobe v4l2loopback".into(),
        ))
    }
}
