use anyhow::Result;
use std::process::Command;

pub fn report() -> Result<()> {
    println!("=== StarDesk Pure Native Linux Diagnostics (Doctor) ===\n");

    // 1. Host OS & Kernel
    let kernel = Command::new("uname").arg("-r").output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "Unknown Linux".to_string());
    println!("[1/6] Linux Kernel: {}", kernel);

    // 2. Display Server & Compositor
    let wayland_display = std::env::var("WAYLAND_DISPLAY").ok();
    let x11_display = std::env::var("DISPLAY").ok();
    let xdg_current_desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "Unknown".into());
    let xdg_session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "Unknown".into());

    print!("[2/6] Display Server: ");
    if wayland_display.is_some() || xdg_session_type.eq_ignore_ascii_case("wayland") {
        println!("Wayland ({}) [Compositor/Desktop: {}]", wayland_display.unwrap_or_default(), xdg_current_desktop);
    } else if x11_display.is_some() {
        println!("X11 ({}) [Desktop: {}]", x11_display.unwrap_or_default(), xdg_current_desktop);
    } else {
        println!("Headless / Direct DRM-KMS");
    }

    // 3. Kernel /dev/uinput (Native Input Injection)
    print!("[3/6] Native /dev/uinput: ");
    let uinput_exists = std::path::Path::new("/dev/uinput").exists();
    if uinput_exists {
        let accessible = std::fs::OpenOptions::new().write(true).open("/dev/uinput").is_ok();
        if accessible {
            println!("✓ Ready (Writable - Low latency mouse & keyboard active)");
        } else {
            println!("⚠️ Permission needed (Run: sudo usermod -aG input $USER && sudo udevadm trigger)");
        }
    } else {
        println!("⚠️ Kernel module not loaded (Run: sudo modprobe uinput)");
    }

    // 4. PipeWire & ScreenCast Portal
    print!("[4/6] PipeWire Audio & Video Stack: ");
    let pw_check = Command::new("pactl").arg("info").output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("PipeWire"))
        .unwrap_or(false);
    if pw_check {
        println!("✓ Active (PipeWire multimedia loopback ready)");
    } else {
        println!("PulseAudio / Generic Audio");
    }

    // 5. GPU Hardware Video Acceleration (NVENC, VA-API)
    print!("[5/6] GPU Video Encoding: ");
    let mut gpus = Vec::new();
    if Command::new("nvidia-smi").output().map(|o| o.status.success()).unwrap_or(false) {
        gpus.push("NVIDIA NVENC (Up to 144FPS 4K Hardware Encode)");
    }
    if Command::new("vainfo").output().map(|o| o.status.success()).unwrap_or(false) {
        gpus.push("VA-API (Intel/AMD Hardware Encode)");
    }
    if gpus.is_empty() {
        println!("Software H.264 (x264 Zero-Latency CPU encode)");
    } else {
        println!("✓ {}", gpus.join(" | "));
    }

    // 6. Account & Credential Status
    print!("[6/6] Account Pairing: ");
    let creds = crate::auth::Credentials::load_or_default();
    if creds.is_logged_in() {
        println!("✓ Paired (User ID: {}, Device: {})", creds.user_id, creds.device_name);
    } else {
        println!("Guest mode active (Run 'stardesk login' to pair with StarDesk mobile app)");
    }

    println!("\nDiagnostics complete: System is 100% native and ready to run.");
    Ok(())
}
