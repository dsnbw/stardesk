use anyhow::{Context, Result};
use std::process::{Command, Stdio};
use std::path::Path;

use crate::config::wine_prefix;
use crate::wine::{find_stardesk_server_exe, find_stardesk_gui_exe};

pub fn start() -> Result<()> {
    let prefix = wine_prefix()?;
    if !prefix.exists() {
        println!("Wine prefix not initialized. Running 'stardesk setup' first...");
        crate::wine::setup(true, false, None)?;
    }

    println!("Starting StarDesk Linux Companion Session...");

    let self_exe = std::env::current_exe()?;

    // 1. Spawn input bridge daemon
    println!("Starting input bridge daemon...");
    let mut bridge = Command::new(&self_exe)
        .arg("__bridge")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("spawning input bridge")?;

    // 2. Spawn screen capture daemon
    println!("Starting video capture service...");
    let mut capture = Command::new(&self_exe)
        .arg("__capture")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("spawning video capture")?;

    // Give daemons a moment to bind sockets & shared memory
    std::thread::sleep(std::time::Duration::from_millis(500));

    // 3. Launch StarDesk Server / Host streamer in Wine
    if let Some(server_exe) = find_stardesk_server_exe(&prefix) {
        println!("Launching StarDesk streaming host: {}", server_exe.display());
        let _ = Command::new("wine")
            .env("WINEPREFIX", &prefix)
            .env("WINEDEBUG", "-all")
            .arg(server_exe.to_string_lossy().as_ref())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }

    // 4. Launch StarDesk Client GUI
    if let Some(gui_exe) = find_stardesk_gui_exe(&prefix) {
        println!("Launching StarDesk GUI Client: {}", gui_exe.display());
        let mut gui = Command::new("wine")
            .env("WINEPREFIX", &prefix)
            .env("WINEDEBUG", "-all")
            .arg(gui_exe.to_string_lossy().as_ref())
            .spawn()
            .context("launching StarDesk GUI")?;

        // Wait for GUI to exit
        let _ = gui.wait();
    } else {
        println!("StarDesk executable not found. Please verify installation with 'stardesk doctor'.");
    }

    // Cleanup background daemons on exit
    let _ = bridge.kill();
    let _ = capture.kill();
    println!("StarDesk session closed.");
    Ok(())
}

pub fn stop() -> Result<()> {
    println!("Stopping all StarDesk Linux companion and Wine processes...");
    let prefix = wine_prefix()?;
    if prefix.exists() {
        let _ = Command::new("wineserver")
            .env("WINEPREFIX", &prefix)
            .arg("-k")
            .status();
    }
    let _ = Command::new("pkill").args(["-f", "stardesk __bridge"]).status();
    let _ = Command::new("pkill").args(["-f", "stardesk __capture"]).status();
    println!("All StarDesk processes stopped.");
    Ok(())
}
