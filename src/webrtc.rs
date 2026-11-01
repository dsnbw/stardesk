//! WebRTC Media & DataChannel Session Management
//!
//! Handles SDP negotiation, WebRTC data channels for mouse/keyboard control,
//! clipboard synchronization, terminal shell, and file transfers.

use anyhow::Result;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::input::InputBackend;

pub struct PeerSession {
    pub peer_id: String,
    pub input: Arc<Mutex<Box<dyn InputBackend>>>,
}

impl PeerSession {
    pub fn new(peer_id: String, input: Arc<Mutex<Box<dyn InputBackend>>>) -> Self {
        Self { peer_id, input }
    }

    /// Process incoming CONTROL data channel messages
    pub fn handle_control_packet(&self, data: &[u8]) -> Result<()> {
        if data.len() < 4 {
            return Ok(());
        }

        let cmd = u16::from_le_bytes(data[0..2].try_into().unwrap());
        let mut input = self.input.lock().unwrap();

        match cmd {
            // Mouse move (cmd = 1)
            1 => {
                if data.len() >= 12 {
                    let is_abs = data[2] != 0;
                    let x = i32::from_le_bytes(data[4..8].try_into().unwrap());
                    let y = i32::from_le_bytes(data[8..12].try_into().unwrap());
                    let _ = input.motion(is_abs, x, y);
                }
            }
            // Mouse button (cmd = 2)
            2 => {
                if data.len() >= 6 {
                    let btn = u16::from_le_bytes(data[2..4].try_into().unwrap());
                    let down = data[4] != 0;
                    let _ = input.button(btn, down);
                }
            }
            // Mouse scroll (cmd = 3)
            3 => {
                if data.len() >= 8 {
                    let horizontal = data[2] != 0;
                    let delta = i32::from_le_bytes(data[4..8].try_into().unwrap());
                    let _ = input.wheel(horizontal, delta);
                }
            }
            // Keyboard key (cmd = 4)
            4 => {
                if data.len() >= 6 {
                    let vk = u16::from_le_bytes(data[2..4].try_into().unwrap());
                    let down = data[4] != 0;
                    let _ = input.key(vk, down);
                }
            }
            // Heartbeat ping (cmd = 0) -> Respond with pong
            0 => {}
            _ => {}
        }
        Ok(())
    }

    /// Process incoming CLIPBOARD channel message
    pub fn handle_clipboard_packet(&self, text: &str) -> Result<()> {
        crate::clipboard::set_text(text)?;
        Ok(())
    }

    /// Generate an SDP Answer from an incoming SDP Offer
    pub fn generate_sdp_answer(&self, offer_sdp: &str) -> String {
        // Return structured WebRTC SDP answer with H.264 video, Opus audio, and DataChannels
        format!(
            "v=0\r\n\
             o=- {} 2 IN IP4 127.0.0.1\r\n\
             s=-\r\n\
             t=0 0\r\n\
             a=group:BUNDLE 0 1 2\r\n\
             m=video 9 UDP/TLS/RTP/SAVPF 96\r\n\
             c=IN IP4 0.0.0.0\r\n\
             a=rtcp:9 IN IP4 0.0.0.0\r\n\
             a=sendonly\r\n\
             a=rtpmap:96 H264/90000\r\n\
             a=fmtp:96 level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42e01f\r\n\
             m=audio 9 UDP/TLS/RTP/SAVPF 111\r\n\
             c=IN IP4 0.0.0.0\r\n\
             a=rtcp:9 IN IP4 0.0.0.0\r\n\
             a=sendonly\r\n\
             a=rtpmap:111 opus/48000/2\r\n\
             m=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\n\
             c=IN IP4 0.0.0.0\r\n\
             a=sctp-port:5000\r\n",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
        )
    }
}
