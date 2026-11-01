use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::config::{data_dir, wine_prefix, Config};

const OFFICIAL_DOWNLOAD_URL: &str = "https://d1k387l9eocp8r.cloudfront.net/StarDesk-1.5.4.exe";
const EULA_URL: &str = "https://www.stardesk.com/eula";

pub fn setup(accept_eula: bool, repair: bool, installer_path: Option<&Path>) -> Result<()> {
    let mut config = Config::load()?;

    if !config.eula_accepted {
        if !accept_eula {
            println!("StarDesk End User License Agreement: {}", EULA_URL);
            println!("Please re-run with --accept-eula to proceed with installation.");
            anyhow::bail!("EULA acceptance required");
        }
        config.eula_accepted = true;
        config.store()?;
        println!("EULA accepted.");
    }

    let prefix = wine_prefix()?;
    std::fs::create_dir_all(&prefix).context("creating wine prefix directory")?;

    let cache = data_dir()?.join("cache");
    std::fs::create_dir_all(&cache).context("creating cache directory")?;

    println!("Initializing Wine prefix at {}...", prefix.display());
    let status = Command::new("wineboot")
        .env("WINEPREFIX", &prefix)
        .env("WINEDEBUG", "-all")
        .arg(if repair { "--update" } else { "--init" })
        .status()
        .context("running wineboot")?;

    if !status.success() {
        anyhow::bail!("wineboot failed with status {}", status);
    }

    // Check if client is already installed
    let client_dir = find_stardesk_dir(&prefix);
    if !repair && client_dir.is_some() {
        finish_setup(&prefix)?;
        println!("StarDesk is already installed in prefix.");
        return Ok(());
    }

    // Determine installer
    let installer = if let Some(path) = installer_path {
        path.to_path_buf()
    } else {
        let cached = cache.join("StarDesk-Setup.exe");
        if !cached.exists() {
            println!("Downloading official StarDesk installer...");
            let status = Command::new("curl")
                .args([
                    "-fL",
                    "--retry", "3",
                    "--progress-bar",
                    "-o", cached.to_string_lossy().as_ref(),
                    OFFICIAL_DOWNLOAD_URL,
                ])
                .status()
                .context("running curl to download StarDesk installer")?;
            if !status.success() {
                anyhow::bail!("Failed to download StarDesk installer from {}", OFFICIAL_DOWNLOAD_URL);
            }
        }
        cached
    };

    println!("Installing StarDesk silently inside Wine prefix...");
    let status = Command::new("wine")
        .env("WINEPREFIX", &prefix)
        .env("WINEDEBUG", "-all")
        .arg(installer.to_string_lossy().as_ref())
        .arg("/S")
        .status()
        .context("executing StarDesk installer")?;

    if !status.success() {
        anyhow::bail!("StarDesk installer failed with status {}", status);
    }

    finish_setup(&prefix)?;
    println!("StarDesk installation and compatibility tuning completed successfully!");
    Ok(())
}

pub fn finish_setup(prefix: &Path) -> Result<()> {
    println!("Applying StarDesk registry compatibility settings...");
    apply_registry_compat(prefix)?;
    deploy_hook_binaries(prefix)?;
    Ok(())
}

pub fn find_stardesk_dir(prefix: &Path) -> Option<PathBuf> {
    let drive_c = prefix.join("drive_c");
    let candidates = [
        drive_c.join("Program Files").join("StarDesk"),
        drive_c.join("Program Files (x86)").join("StarDesk"),
        drive_c.join("Program Files").join("Netease").join("GameViewer"),
        drive_c.join("Program Files (x86)").join("Netease").join("GameViewer"),
    ];

    for candidate in candidates {
        if candidate.join("StarDesk.exe").exists()
            || candidate.join("bin").join("StarDeskServer.exe").exists()
            || candidate.join("GameViewer.exe").exists()
            || candidate.join("bin").join("GameViewerServer.exe").exists()
        {
            return Some(candidate);
        }
    }
    None
}

pub fn find_stardesk_server_exe(prefix: &Path) -> Option<PathBuf> {
    let root = find_stardesk_dir(prefix)?;
    let candidates = [
        root.join("bin").join("StarDeskServer.exe"),
        root.join("StarDeskServer.exe"),
        root.join("bin").join("GameViewerServer.exe"),
        root.join("GameViewerServer.exe"),
    ];
    candidates.into_iter().find(|p| p.exists())
}

pub fn find_stardesk_gui_exe(prefix: &Path) -> Option<PathBuf> {
    let root = find_stardesk_dir(prefix)?;
    let candidates = [
        root.join("StarDesk.exe"),
        root.join("bin").join("StarDesk.exe"),
        root.join("GameViewer.exe"),
        root.join("bin").join("GameViewer.exe"),
    ];
    candidates.into_iter().find(|p| p.exists())
}

fn apply_registry_compat(prefix: &Path) -> Result<()> {
    let entries = [
        // Edge WebView2 requires Windows 8 mode under Wine
        (r"HKCU\Software\Wine\AppDefaults\msedgewebview2.exe", "Version", "REG_SZ", "win8"),
        // StarDeskServer requires Windows 10 mode for ConPTY support
        (r"HKCU\Software\Wine\AppDefaults\StarDeskServer.exe", "Version", "REG_SZ", "win10"),
        (r"HKCU\Software\Wine\AppDefaults\GameViewerServer.exe", "Version", "REG_SZ", "win10"),
        // Use native C++ runtime
        (r"HKCU\Software\Wine\AppDefaults\StarDesk.exe\DllOverrides", "msvcp140", "REG_SZ", "native,builtin"),
        (r"HKCU\Software\Wine\AppDefaults\StarDeskServer.exe\DllOverrides", "msvcp140", "REG_SZ", "native,builtin"),
        (r"HKCU\Software\Wine\AppDefaults\GameViewer.exe\DllOverrides", "msvcp140", "REG_SZ", "native,builtin"),
        (r"HKCU\Software\Wine\AppDefaults\GameViewerServer.exe\DllOverrides", "msvcp140", "REG_SZ", "native,builtin"),
        // Service auto start
        (r"HKLM\System\CurrentControlSet\Services\StarDeskService", "Start", "REG_DWORD", "2"),
        (r"HKLM\System\CurrentControlSet\Services\GameViewerService", "Start", "REG_DWORD", "2"),
        // DLL overrides for hooks
        (r"HKCU\Software\Wine\DllOverrides", "wevtapi", "REG_SZ", "native"),
        (r"HKCU\Software\Wine\DllOverrides", "wtsapi32", "REG_SZ", "native"),
        (r"HKCU\Software\Wine\DllOverrides", "winemenubuilder", "REG_SZ", ""),
    ];

    for (key, val, reg_type, data) in entries {
        let _ = Command::new("wine")
            .env("WINEPREFIX", prefix)
            .env("WINEDEBUG", "-all")
            .args(["reg", "add", key, "/v", val, "/t", reg_type, "/d", data, "/f"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    Ok(())
}

fn deploy_hook_binaries(prefix: &Path) -> Result<()> {
    let system32 = prefix.join("drive_c").join("windows").join("system32");
    std::fs::create_dir_all(&system32)?;

    // Copy hook DLLs if found in executable directory or assets
    let exe_dir = std::env::current_exe()?
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let hook_candidates = [
        exe_dir.join("hook"),
        exe_dir.join("../lib/stardesk/hook"),
        PathBuf::from("/usr/lib/stardesk/hook"),
        PathBuf::from("/usr/local/lib/stardesk/hook"),
    ];

    for candidate in hook_candidates {
        if candidate.join("stardesk-hook.dll").exists() {
            let files = ["stardesk-hook.dll", "wevtapi.dll", "wtsapi32.dll"];
            for file in files {
                let src = candidate.join(file);
                if src.exists() {
                    let _ = std::fs::copy(&src, system32.join(file));
                    if let Some(stardesk_dir) = find_stardesk_dir(prefix) {
                        let _ = std::fs::copy(&src, stardesk_dir.join(file));
                        let _ = std::fs::copy(&src, stardesk_dir.join("bin").join(file));
                    }
                }
            }
            break;
        }
    }
    Ok(())
}
