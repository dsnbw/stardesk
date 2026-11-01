//! StarDesk & UU Remote Native Authentication & REST API Client
//!
//! Handles HMAC-SHA256 request signing, QR code login, device registration,
//! signaling server room allocation, and session heartbeats.

use anyhow::{Context, Result};
use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

pub const DEFAULT_SIGN_KEY: &str = "alWiSzXZTLu3WfFnw13uBru3";
pub const BASE_URL: &str = "https://api.nrd.nie.163.com";

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct Credentials {
    pub user_id: String,
    pub device_id: String,
    pub client_id: String,
    pub system_id: String,
    pub token: String,
    pub csrf: String,
    pub sign_key: String,
    pub device_name: String,
}

impl Credentials {
    pub fn load_or_default() -> Self {
        let path = crate::config::credentials_path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(creds) = serde_json::from_str(&data) {
                return creds;
            }
        }
        Self {
            sign_key: DEFAULT_SIGN_KEY.to_string(),
            client_id: uuid_string(),
            system_id: uuid_string(),
            device_id: uuid_string(),
            device_name: get_hostname(),
            ..Default::default()
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = crate::config::credentials_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, data)?;
        Ok(())
    }

    pub fn is_logged_in(&self) -> bool {
        !self.token.is_empty() && !self.user_id.is_empty()
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QrCodeData {
    pub ticket: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QrPollData {
    pub status: i32, // 0 = waiting, 1 = scanned, 2 = confirmed, 3 = expired
    pub user_id: Option<String>,
    pub token: Option<String>,
    pub csrf: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RoomData {
    pub signaling_server: String,
    pub token: String,
    pub signaling_list: Option<Vec<String>>,
}

pub struct StarDeskAuthClient {
    pub creds: Credentials,
    http: reqwest::Client,
}

impl StarDeskAuthClient {
    pub fn new(creds: Credentials) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Self { creds, http }
    }

    pub fn sign_request(
        &self,
        method: &str,
        path: &str,
        params: &BTreeMap<String, String>,
        body: &str,
    ) -> String {
        let mut sign_parts = Vec::new();
        sign_parts.push(method.to_uppercase());
        sign_parts.push(path.to_string());

        let mut param_str = String::new();
        for (k, v) in params {
            if !param_str.is_empty() {
                param_str.push('&');
            }
            param_str.push_str(&format!("{}={}", k, v));
        }
        sign_parts.push(param_str);
        if !body.is_empty() {
            sign_parts.push(body.to_string());
        }

        let payload = sign_parts.join("\n");
        let mut mac = HmacSha256::new_from_slice(self.creds.sign_key.as_bytes())
            .expect("HMAC initialization failed");
        mac.update(payload.as_bytes());
        let result = mac.finalize();
        hex_encode(&result.into_bytes())
    }

    pub fn build_headers(&self, method: &str, path: &str, body: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut params = BTreeMap::new();
        params.insert("x-param-client-id".into(), self.creds.client_id.clone());
        params.insert("x-param-device-id".into(), self.creds.device_id.clone());
        params.insert("x-param-nonce".into(), random_hex(16));
        params.insert("x-param-plat".into(), "1".into()); // 1 = PC Client
        params.insert("x-param-timestamp".into(), now.to_string());
        params.insert("x-param-ver".into(), "1.5.4".into());

        if !self.creds.user_id.is_empty() {
            params.insert("x-param-user-id".into(), self.creds.user_id.clone());
        }

        let signature = self.sign_request(method, path, &params, body);
        params.insert("x-param-sign".into(), signature);

        for (k, v) in params {
            if let (Ok(name), Ok(val)) = (HeaderName::from_bytes(k.as_bytes()), HeaderValue::from_str(&v)) {
                headers.insert(name, val);
            }
        }

        if !self.creds.token.is_empty() {
            headers.insert("authorization", HeaderValue::from_str(&format!("Bearer {}", self.creds.token)).unwrap());
        }
        if !self.creds.csrf.is_empty() {
            headers.insert("x-csrf-token", HeaderValue::from_str(&self.creds.csrf).unwrap());
        }

        headers.insert("content-type", HeaderValue::from_static("application/json"));
        headers
    }

    /// Request a new QR Code for account login
    pub async fn fetch_qr_code(&self) -> Result<QrCodeData> {
        let path = "/api/v1/auth/qr/create";
        let url = format!("{}{}", BASE_URL, path);
        let headers = self.build_headers("POST", path, "{}");

        let res = self.http.post(&url)
            .headers(headers)
            .body("{}")
            .send()
            .await
            .context("sending qr create request")?;

        let json: ApiResponse<QrCodeData> = res.json().await.context("parsing qr response")?;
        if json.code != 0 || json.data.is_none() {
            anyhow::bail!("Failed to generate QR: {} (code: {})", json.msg, json.code);
        }
        Ok(json.data.unwrap())
    }

    /// Poll QR Code scan status
    pub async fn poll_qr_status(&self, ticket: &str) -> Result<QrPollData> {
        let path = "/api/v1/auth/qr/poll";
        let body = serde_json::json!({ "ticket": ticket }).to_string();
        let url = format!("{}{}", BASE_URL, path);
        let headers = self.build_headers("POST", path, &body);

        let res = self.http.post(&url)
            .headers(headers)
            .body(body)
            .send()
            .await
            .context("polling qr status")?;

        let json: ApiResponse<QrPollData> = res.json().await.context("parsing poll response")?;
        if json.code != 0 || json.data.is_none() {
            anyhow::bail!("Failed to poll QR: {} (code: {})", json.msg, json.code);
        }
        Ok(json.data.unwrap())
    }

    /// Register or update device information in user's device list
    pub async fn register_device(&self) -> Result<()> {
        let path = "/api/v1/device/register";
        let body = serde_json::json!({
            "device_name": self.creds.device_name,
            "os_type": "Linux",
            "os_version": get_os_info(),
            "client_version": "1.5.4",
            "device_id": self.creds.device_id,
        }).to_string();

        let url = format!("{}{}", BASE_URL, path);
        let headers = self.build_headers("POST", path, &body);

        let res = self.http.post(&url)
            .headers(headers)
            .body(body)
            .send()
            .await?;

        let json: ApiResponse<serde_json::Value> = res.json().await?;
        if json.code != 0 {
            anyhow::bail!("Device registration failed: {}", json.msg);
        }
        Ok(())
    }

    /// Allocate signaling server room for remote streaming
    pub async fn allocate_room(&self) -> Result<RoomData> {
        let path = "/api/v1/room/create";
        let body = serde_json::json!({
            "device_id": self.creds.device_id,
            "role": "host",
        }).to_string();

        let url = format!("{}{}", BASE_URL, path);
        let headers = self.build_headers("POST", path, &body);

        let res = self.http.post(&url)
            .headers(headers)
            .body(body)
            .send()
            .await?;

        let json: ApiResponse<RoomData> = res.json().await?;
        if json.code != 0 || json.data.is_none() {
            anyhow::bail!("Failed to create streaming room: {}", json.msg);
        }
        Ok(json.data.unwrap())
    }

    /// Send heartbeat to keep device listed as Online
    pub async fn send_heartbeat(&self) -> Result<()> {
        let path = "/api/v1/device/heartbeat";
        let body = serde_json::json!({ "device_id": self.creds.device_id }).to_string();
        let url = format!("{}{}", BASE_URL, path);
        let headers = self.build_headers("POST", path, &body);

        let _ = self.http.post(&url).headers(headers).body(body).send().await?;
        Ok(())
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn random_hex(len: usize) -> String {
    let mut bytes = vec![0u8; len / 2];
    for b in bytes.iter_mut() {
        *b = (std::time::SystemTime::now().elapsed().unwrap().as_nanos() & 0xff) as u8;
    }
    hex_encode(&bytes)
}

fn uuid_string() -> String {
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (std::time::SystemTime::now().elapsed().unwrap().as_nanos() & 0xffffffff) as u32,
        (std::time::SystemTime::now().elapsed().unwrap().as_micros() & 0xffff) as u16,
        (std::time::SystemTime::now().elapsed().unwrap().as_nanos() & 0x0fff) as u16,
        0x8000 | ((std::time::SystemTime::now().elapsed().unwrap().as_micros() & 0x3fff) as u16),
        (std::time::SystemTime::now().elapsed().unwrap().as_nanos() & 0xffffffffffff) as u64
    )
}

fn get_hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::fs::read_to_string("/etc/hostname").map(|s| s.trim().to_string()))
        .unwrap_or_else(|_| "Linux-StarDesk".to_string())
}

fn get_os_info() -> String {
    let uname = std::process::Command::new("uname").arg("-sr").output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "Linux".to_string());
    uname
}
