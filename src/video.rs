use anyhow::Result;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy)]
pub enum VideoEncoder {
    Nvenc,      // NVIDIA Hardware Encoder
    Vaapi,      // Intel / AMD VA-API Hardware Encoder
    Software,   // x264 ultrafast software encoder
}

pub struct EncoderPipeline {
    pub encoder: VideoEncoder,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate_kbps: u32,
}

impl EncoderPipeline {
    pub fn probe() -> VideoEncoder {
        // Probe for NVIDIA NVENC
        if Command::new("nvidia-smi").output().map(|o| o.status.success()).unwrap_or(false) {
            println!("Detected NVIDIA GPU - Using NVENC Hardware Encoder");
            return VideoEncoder::Nvenc;
        }

        // Probe for VA-API
        if Command::new("vainfo").output().map(|o| o.status.success()).unwrap_or(false) {
            println!("Detected VA-API Driver - Using VA-API Hardware Encoder");
            return VideoEncoder::Vaapi;
        }

        println!("Using optimized Software H.264 Encoder (Zero-Latency Preset)");
        VideoEncoder::Software
    }

    pub fn new(width: u32, height: u32, fps: u32) -> Self {
        let encoder = Self::probe();
        Self {
            encoder,
            width,
            height,
            fps,
            bitrate_kbps: 8000,
        }
    }
}
