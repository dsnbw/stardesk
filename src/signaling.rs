//! StarDesk Signaling Protocol & WebSocket Client
//!
//! Communicates with the StarDesk/NetEase signaling server for room admission,
//! WebRTC SDP exchange, ICE candidate discovery, and session state sync.

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct SignalingPacket {
    pub action: String,
    pub room_id: Option<String>,
    pub from_peer: Option<String>,
    pub to_peer: Option<String>,
    pub sdp: Option<String>,
    pub sdp_type: Option<String>,
    pub candidate: Option<String>,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<u32>,
    pub data: Option<serde_json::Value>,
}

pub struct SignalingClient {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    pub room_token: String,
}

impl SignalingClient {
    pub async fn connect(url: &str, token: &str) -> Result<Self> {
        let full_url = if url.contains('?') {
            format!("{}&token={}", url, token)
        } else {
            format!("{}?token={}", url, token)
        };

        println!("Connecting to signaling server at: {}", url);
        let (ws, response) = connect_async(&full_url)
            .await
            .with_context(|| format!("failed to connect to signaling server at {}", url))?;

        println!("Connected to signaling server (HTTP {})", response.status());

        Ok(Self {
            ws,
            room_token: token.to_string(),
        })
    }

    /// Join host room
    pub async fn join_host(&mut self, device_id: &str) -> Result<()> {
        let msg = SignalingPacket {
            action: "join".into(),
            room_id: None,
            from_peer: Some(device_id.to_string()),
            to_peer: None,
            sdp: None,
            sdp_type: None,
            candidate: None,
            sdp_mid: None,
            sdp_mline_index: None,
            data: Some(serde_json::json!({
                "role": "host",
                "device_id": device_id,
                "platform": "linux",
                "client_ver": "1.5.4"
            })),
        };

        let json = serde_json::to_string(&msg)?;
        self.ws.send(Message::Text(json)).await?;
        Ok(())
    }

    /// Send SDP Answer
    pub async fn send_sdp_answer(&mut self, to_peer: &str, sdp: &str) -> Result<()> {
        let msg = SignalingPacket {
            action: "sdp".into(),
            room_id: None,
            from_peer: None,
            to_peer: Some(to_peer.to_string()),
            sdp: Some(sdp.to_string()),
            sdp_type: Some("answer".into()),
            candidate: None,
            sdp_mid: None,
            sdp_mline_index: None,
            data: None,
        };
        let json = serde_json::to_string(&msg)?;
        self.ws.send(Message::Text(json)).await?;
        Ok(())
    }

    /// Send ICE candidate
    pub async fn send_ice_candidate(
        &mut self,
        to_peer: &str,
        candidate: &str,
        mid: Option<&str>,
        mline: Option<u32>,
    ) -> Result<()> {
        let msg = SignalingPacket {
            action: "candidate".into(),
            room_id: None,
            from_peer: None,
            to_peer: Some(to_peer.to_string()),
            sdp: None,
            sdp_type: None,
            candidate: Some(candidate.to_string()),
            sdp_mid: mid.map(str::to_string),
            sdp_mline_index: mline,
            data: None,
        };
        let json = serde_json::to_string(&msg)?;
        self.ws.send(Message::Text(json)).await?;
        Ok(())
    }

    /// Receive next signaling message
    pub async fn next_message(&mut self) -> Option<Result<SignalingPacket>> {
        while let Some(msg) = self.ws.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<SignalingPacket>(&text) {
                        Ok(packet) => return Some(Ok(packet)),
                        Err(e) => eprintln!("Failed to parse signaling packet: {}", e),
                    }
                }
                Ok(Message::Ping(payload)) => {
                    let _ = self.ws.send(Message::Pong(payload)).await;
                }
                Ok(Message::Close(_)) => return None,
                Err(e) => return Some(Err(anyhow::anyhow!("websocket error: {}", e))),
                _ => {}
            }
        }
        None
    }
}
