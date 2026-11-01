use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub eula_accepted: bool,
    pub capture_backend: String, // "portal", "wayland", "x11", "drm"
    pub input_backend: String,   // "uinput", "xtest", "auto"
    pub capture_source: String,  // "monitor", "window", "virtual"
    pub capture_output: Option<String>,
    pub hardware_accel: bool,
    pub max_fps: u32,            // 30, 60, 90, 144
    pub video_bitrate_kbps: u32, // 2000..20000
    pub allow_remote_assistance: bool,
    pub bridge_port: u16,
    pub launcher_port: u16,
    pub terminal_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            eula_accepted: true,
            capture_backend: "portal".to_string(),
            input_backend: "auto".to_string(),
            capture_source: "monitor".to_string(),
            capture_output: None,
            hardware_accel: true,
            max_fps: 60,
            video_bitrate_kbps: 8000,
            allow_remote_assistance: true,
            bridge_port: 54192,
            launcher_port: 54193,
            terminal_port: 54194,
        }
    }
}

impl Config {
    pub fn config_path() -> Result<PathBuf> {
        let dir = config_dir()?;
        Ok(dir.join("config.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = std::fs::read_to_string(&path)
            .with_context(|| format!("reading config file at {}", path.display()))?;
        let config: Self = serde_json::from_str(&data).unwrap_or_default();
        Ok(config)
    }

    pub fn store(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, data)?;
        Ok(())
    }
}

pub fn config_dir() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(dir).join("stardesk"));
    }
    let home = std::env::var_os("HOME").context("HOME directory not set")?;
    Ok(PathBuf::from(home).join(".config").join("stardesk"))
}

pub fn credentials_path() -> PathBuf {
    config_dir().unwrap_or_else(|_| PathBuf::from(".")).join("credentials.json")
}

pub fn data_dir() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os("XDG_DATA_HOME") {
        return Ok(PathBuf::from(dir).join("stardesk"));
    }
    let home = std::env::var_os("HOME").context("HOME directory not set")?;
    Ok(PathBuf::from(home).join(".local").join("share").join("stardesk"))
}

pub fn wine_prefix() -> Result<PathBuf> {
    Ok(data_dir()?.join("wine"))
}
