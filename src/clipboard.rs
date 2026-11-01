use anyhow::Result;
use std::process::{Command, Stdio};
use std::io::Write;

pub fn get_text() -> Result<String> {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        if let Ok(out) = Command::new("wl-paste").arg("--no-newline").output() {
            if out.status.success() {
                return Ok(String::from_utf8_lossy(&out.stdout).to_string());
            }
        }
    }

    if let Ok(out) = Command::new("xclip").args(["-selection", "clipboard", "-o"]).output() {
        if out.status.success() {
            return Ok(String::from_utf8_lossy(&out.stdout).to_string());
        }
    }

    Ok(String::new())
}

pub fn set_text(text: &str) -> Result<()> {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        if let Ok(mut child) = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
            return Ok(());
        }
    }

    if let Ok(mut child) = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
        return Ok(());
    }

    Ok(())
}
