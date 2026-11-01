use anyhow::Result;

pub async fn connect_to_remote(target_device_id: &str) -> Result<()> {
    println!("Connecting to remote StarDesk host: {}", target_device_id);
    let creds = crate::auth::Credentials::load_or_default();
    if !creds.is_logged_in() {
        println!("Notice: You are connecting in Guest / Assistance mode.");
    }

    let auth = crate::auth::StarDeskAuthClient::new(creds);
    println!("Allocating remote session route...");
    // Viewer session lifecycle
    Ok(())
}
