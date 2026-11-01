use anyhow::Result;
use std::process::{Command, Stdio};

pub struct AudioCapture {
    pub sample_rate: u32,
    pub channels: u16,
}

impl AudioCapture {
    pub fn new() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
        }
    }

    /// Check if PipeWire or PulseAudio is available
    pub fn is_available() -> bool {
        Command::new("pactl")
            .arg("info")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
