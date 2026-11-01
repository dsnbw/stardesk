use anyhow::{Context, Result};
use std::path::PathBuf;

fn autostart_dir() -> Result<PathBuf> {
    let home = std::env::var_os("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join(".config").join("autostart"))
}

pub fn enable() -> Result<()> {
    let dir = autostart_dir()?;
    std::fs::create_dir_all(&dir)?;
    let desktop_file = dir.join("stardesk.desktop");

    let exe = std::env::current_exe()?;
    let content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=StarDesk\n\
         Comment=StarDesk Remote Desktop\n\
         Exec={} run\n\
         Icon=stardesk\n\
         Terminal=false\n\
         Categories=Network;RemoteAccess;\n\
         X-GNOME-Autostart-enabled=true\n",
        exe.display()
    );

    std::fs::write(&desktop_file, content)?;
    println!("StarDesk autostart enabled: {}", desktop_file.display());
    Ok(())
}

pub fn disable() -> Result<()> {
    let desktop_file = autostart_dir()?.join("stardesk.desktop");
    if desktop_file.exists() {
        std::fs::remove_file(&desktop_file)?;
        println!("StarDesk autostart disabled.");
    } else {
        println!("Autostart entry was not enabled.");
    }
    Ok(())
}

pub fn status() -> Result<()> {
    let desktop_file = autostart_dir()?.join("stardesk.desktop");
    if desktop_file.exists() {
        println!("Autostart: Enabled ({})", desktop_file.display());
    } else {
        println!("Autostart: Disabled");
    }
    Ok(())
}
