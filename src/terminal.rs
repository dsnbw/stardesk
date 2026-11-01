use anyhow::Result;
use std::process::{Command, Stdio};

pub struct TerminalSession {
    pub session_id: String,
}

impl TerminalSession {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }

    /// Spawns a native bash / sh PTY session
    pub fn spawn_shell(&self) -> Result<()> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
        println!("Starting native Linux shell: {}", shell);
        Ok(())
    }
}
